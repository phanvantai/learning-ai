---
title: "Eval cho ứng dụng LLM và agent: bắt đầu từ con số 0"
date: 2026-11-10
tags: [Eval, Agent]
description: "Vì sao thử vài câu thấy ổn là chưa đủ, và cách dựng bộ eval đầu tiên cho agent: thuật ngữ, ba loại grader, pass@k so với pass^k, đọc transcript trước khi viết metric, hiệu chỉnh LLM judge."
loai: chuyen-sau
---

Bạn sửa một dòng trong system prompt để agent bớt dài dòng. Thử năm câu, thấy ổn, deploy. Hai ngày sau có người báo agent hoàn tiền sai chính sách, một lỗi trước đó không có. Bạn không biết lỗi do dòng prompt mới, do lần nâng model tuần trước, hay nó vốn xảy ra 1 trên 20 lần mà trước giờ chưa ai gặp.

Eval sinh ra để trả lời đúng loại câu hỏi đó. Bài này đi từ đầu: thuật ngữ, cách chấm, cách xử lý tính bất định, một harness tối thiểu bằng Python, và những chỗ các nguồn uy tín bất đồng với nhau. Mình dựa chủ yếu vào bài "Demystifying evals for AI agents" của Anthropic, bộ FAQ về eval của Hamel Husain và Shreya Shankar, tài liệu eval của OpenAI và paper τ-bench. Link ở cuối bài.

## Vì sao "thử vài câu thấy ổn" không đủ

Có ba lý do mang tính cơ chế, không phải chuyện cẩn thận hay cẩu thả.

**Đầu ra không tất định.** Cùng một đầu vào, hai lần chạy cho hai kết quả khác nhau. Một lần chạy đúng chỉ nói rằng xác suất đúng lớn hơn 0. Với agent, mỗi bước là một lần lấy mẫu, nên sai lệch nhỏ ở bước 3 có thể dẫn tới một quỹ đạo khác hẳn ở bước 15.

**Lỗi cộng dồn theo số bước.** Một agent làm 20 tool call (lời gọi công cụ), mỗi bước đúng 98% và các bước độc lập, thì cả chuỗi đúng khoảng 0,98^20 ≈ 67%. Thử tay vài lần rất dễ rơi đúng vào phần 67% đó.

**Thay đổi ở một chỗ làm hỏng chỗ khác.** Prompt, mô tả tool, model, cách cắt context đều tác động lên mọi hành vi cùng lúc. Không có bộ kiểm tra chạy lại được thì mỗi lần sửa là một lần đánh đổi mù. Bài của Anthropic mô tả tình trạng này: đội không có eval chỉ biết có lỗi khi người dùng phàn nàn, sửa xong lại không biết có làm hỏng thứ khác không.

Tài liệu của OpenAI gọi anti-pattern này là "vibe-based evals": lấy cảm giác "có vẻ chạy được" làm chiến lược.

## Thuật ngữ: thống nhất trước khi làm

Mình dùng bộ định nghĩa trong bài của Anthropic vì nó tách bạch được những thứ hay bị gọi lẫn.

| Thuật ngữ | Nghĩa |
| --- | --- |
| Task | Một bài kiểm tra: đầu vào xác định và tiêu chí thành công xác định |
| Trial | Một lần chạy task. Chạy nhiều trial vì kết quả dao động giữa các lần |
| Grader | Logic chấm một khía cạnh của kết quả. Một task có thể có nhiều grader |
| Transcript | Bản ghi đầy đủ một trial: đầu ra, tool call, suy luận, kết quả trung gian. Còn gọi là trace hay trajectory |
| Outcome | Trạng thái cuối của môi trường sau trial |
| Eval harness | Hạ tầng chạy eval từ đầu đến cuối: nạp task, chạy song song, ghi lại, chấm, tổng hợp |
| Agent harness | Hệ thống bọc quanh model để nó hành động như agent (vòng lặp, tool, quản lý context) |
| Eval suite | Tập task cùng nhắm một năng lực hay hành vi |

Hai điểm cần giữ trong đầu.

Thứ nhất, transcript và outcome là hai thứ khác nhau. Ví dụ của Anthropic: agent đặt vé nói "đã đặt xong" ở cuối transcript, còn outcome là trong cơ sở dữ liệu có bản ghi đặt chỗ hay không. Agent báo xong mà chưa xong là failure mode rất phổ biến, nên chấm theo lời agent nói là chấm sai chỗ.

Thứ hai, khi eval một agent, bạn đang đo model và agent harness cùng lúc. Đổi cách cắt context hay mô tả tool cũng làm đổi điểm như đổi model. Điểm benchmark công bố của một model vì thế không chuyển thẳng sang hệ thống của bạn được.

## Ba loại grader

| Loại | Ví dụ | Điểm mạnh | Điểm yếu |
| --- | --- | --- | --- |
| Code | So khớp chuỗi, chạy unit test, kiểm tra trạng thái DB, kiểm tra tool nào đã được gọi | Nhanh, rẻ, khách quan, lặp lại được, dễ debug | Giòn trước các biến thể hợp lệ, không bắt được sắc thái |
| Model (LLM-as-judge) | Chấm theo rubric, khẳng định bằng ngôn ngữ tự nhiên, so sánh cặp | Linh hoạt, bắt được sắc thái, xử lý được đầu ra mở | Không tất định, tốn hơn, phải hiệu chỉnh với nhãn người |
| Người | Người am hiểu nghiệp vụ đọc và chấm, spot check, A/B test | Chuẩn vàng, dùng để hiệu chỉnh grader model | Đắt, chậm, khó mở rộng |

(Bảng tóm theo bài của Anthropic.)

Thứ tự ưu tiên thực dụng: cái gì chấm được bằng code thì chấm bằng code. LLM judge chỉ dành cho tiêu chí code không biểu diễn nổi, như "câu trả lời có bám vào tài liệu được cung cấp không" hay "giọng điệu có phù hợp với khách đang bực không". Người chấm là nguồn sự thật để kiểm tra hai loại kia, không phải thứ chạy mỗi lần commit.

Hamel Husain đưa ra một tiêu chí lọc chặt hơn: chỉ viết evaluator tự động cho những lỗi vẫn còn sau khi bạn đã sửa prompt. Lỗi nào sửa một dòng prompt là hết thì không đáng tốn một LLM judge phải bảo trì.

## Capability eval và regression eval

Hai loại này trả lời hai câu hỏi khác nhau và có mức điểm kỳ vọng ngược nhau.

- **Capability eval**: agent làm được gì? Nên bắt đầu ở mức pass thấp để đội có "ngọn đồi để leo". Một capability eval pass 95% ngay từ đầu không cho bạn thông tin gì về hướng cải thiện.
- **Regression eval**: agent còn làm được những việc trước đây đã làm được không? Nên giữ gần 100%. Tụt điểm ở đây nghĩa là có thứ vừa hỏng.

Khi một capability eval bão hòa, các task của nó chuyển sang suite regression, và bạn cần capability eval mới khó hơn. Về vận hành, regression suite chạy trong CI trước mỗi lần deploy, nên ưu tiên grader tất định để kết quả không nhảy lung tung. Capability suite chạy khi thử model mới hay thay kiến trúc, chấp nhận chậm và đắt hơn.

## Chấm kết quả cuối, đừng chấm đường đi

Phản xạ tự nhiên của kỹ sư là kiểm tra agent có gọi đúng chuỗi tool theo đúng thứ tự không. Anthropic khuyên tránh cách này: kiểm tra cứng chuỗi tool call quá giòn, vì agent thường tìm ra cách hợp lệ mà người thiết kế task không lường trước. Bạn sẽ phạt những lời giải đúng.

Chấm outcome nghĩa là sau khi agent chạy xong, bạn nhìn vào môi trường:

- Agent sửa bug: bộ test có pass không. SWE-bench chấm theo kiểu này, mỗi task là một issue thật trên GitHub (2.294 bài toán từ 12 repo Python trong bản gốc) và lời giải được kiểm bằng test của repo.
- Agent chăm sóc khách hàng: trạng thái cơ sở dữ liệu cuối có khớp trạng thái mong đợi không. τ-bench so trạng thái DB cuối phiên với trạng thái đích đã chú thích.
- Agent nghiên cứu: các khẳng định trong báo cáo có được nguồn trích dẫn chống đỡ không. Chỗ này thường phải dùng LLM judge.

Chấm đường đi vẫn có chỗ đứng trong hai trường hợp. Một là ràng buộc an toàn hay chính sách ("không được gọi tool hoàn tiền trước khi xác minh danh tính") vì ở đó đường đi chính là yêu cầu. Hai là chẩn đoán: Hamel Husain đề xuất làm hai pha, pha đầu coi agent là hộp đen và chỉ hỏi nó có đạt mục tiêu của người dùng không, pha sau mới đi vào từng bước khi error analysis đã chỉ ra quy trình nào hỏng nhiều. Công cụ ông gợi ý cho pha sau là "transition failure matrix": hàng là trạng thái thành công cuối cùng, cột là nơi lỗi đầu tiên xuất hiện, để thấy lỗi tụ ở đâu.

Tài liệu của OpenAI thì liệt kê eval ở mức bước cho agent đơn: chọn đúng tool chưa, truyền đúng tham số chưa. Eval từng bước rẻ và dễ viết, nhưng nếu chỉ có chúng, bạn có thể có một agent gọi tool rất chuẩn mà vẫn không xong việc.

Với task nhiều phần, Anthropic gợi ý cho điểm từng phần: agent hỗ trợ xác minh được khách nhưng hoàn tiền hỏng vẫn tốt hơn agent hỏng ngay bước đầu, và điểm số nên phản ánh điều đó.

## Tính bất định: nhiều trial, pass@k và pass^k

Vì mỗi trial cho kết quả khác nhau, một task cần chạy nhiều lần, và bạn phải chọn cách gộp.

- **pass@k**: xác suất có ít nhất một lần đúng trong k lần thử. Tăng theo k.
- **pass^k**: xác suất cả k lần đều đúng. Giảm theo k. Khái niệm này do paper τ-bench đưa ra để đo độ ổn định.

Lấy ví dụ trong bài của Anthropic: tỉ lệ đúng mỗi trial là 75%, chạy 3 trial.

```text
pass@3 = 1 - (1 - 0,75)^3 ≈ 98%
pass^3 = 0,75^3           ≈ 42%
```

Cùng một agent, hai con số kể hai câu chuyện trái ngược. Chọn số nào tùy sản phẩm. Nếu có vòng thử lại hoặc người chọn kết quả tốt nhất (sinh code rồi chạy test, đề xuất nhiều phương án), pass@k phản ánh đúng. Nếu agent đối diện khách hàng và mỗi lần chạy là một lần thật, bạn cần pass^k. Con số trong abstract của τ-bench cho thấy khoảng cách này lớn thế nào: các agent function-calling hàng đầu thời điểm đó (như gpt-4o) thành công dưới 50% task, và pass^8 trong miền bán lẻ dưới 25%.

Vài lưu ý khi chạy nhiều trial:

- **Mỗi trial phải bắt đầu từ môi trường sạch.** Trạng thái dùng chung (file sót lại, cache, lịch sử git) làm các trial tương quan với nhau. Anthropic kể một trường hợp Claude đạt lợi thế không công bằng vì đọc được lịch sử git của các trial trước.
- **Mẫu nhỏ thì sai số lớn.** Với 30 task và tỉ lệ pass 70%, sai số chuẩn khoảng 8 điểm phần trăm, khoảng tin cậy 95% rộng cỡ ±16 điểm. Chênh 5 điểm giữa hai phiên bản prompt trên bộ đó chưa nói lên điều gì. Bộ nhỏ vẫn hữu ích ở giai đoạn đầu vì các thay đổi lúc đó thường tạo hiệu ứng lớn, nhưng đừng dùng nó để phân xử khác biệt nhỏ.
- **Ghi lại cả chi phí.** Số token, số bước, thời gian mỗi trial. Hai phiên bản cùng pass 80% nhưng một bên tốn gấp ba số bước là hai phiên bản khác nhau.

## Bắt đầu: 20-50 task lấy từ lỗi thật

Lý do nhiều đội trì hoãn eval là tưởng cần vài trăm task. Anthropic nói ngược lại: 20-50 task đơn giản rút từ lỗi thật là đủ để bắt đầu. Nguồn task theo thứ tự dễ lấy:

1. Những thứ bạn vẫn kiểm tra tay trước mỗi lần release. Đó đã là eval, chỉ là chưa tự động.
2. Bug tracker và phiếu hỗ trợ: mỗi lỗi người dùng báo là một task có sẵn đầu vào và kết quả sai.
3. Log production. OpenAI khuyên log mọi thứ ngay từ khi phát triển để về sau đào ra test case.

Mỗi task cần ba thứ: đầu vào tái lập được, tiêu chí thành công không mơ hồ, và một lời giải tham chiếu. Phép thử Anthropic đưa ra cho tiêu chí: hai người am hiểu nghiệp vụ độc lập phải ra cùng phán quyết pass/fail. Lời giải tham chiếu chứng minh hai điều cùng lúc: task giải được và grader chấm đúng lời giải đúng.

Cân bằng bộ task theo cả hai chiều. Nếu chỉ có task "agent nên tìm kiếm web", bạn sẽ tối ưu ra một agent tìm kiếm mọi lúc. Anthropic kể bộ eval web search của họ phải phủ cả trường hợp kích hoạt thừa và kích hoạt thiếu, và mất nhiều vòng tinh chỉnh.

## Một eval harness tối thiểu

Đoạn dưới là khung sườn để minh họa cấu trúc, chưa phải thứ dùng thẳng cho production. `run_agent` là chỗ bạn nối agent của mình vào; nó nhận prompt và một thư mục làm việc riêng, trả về transcript.

```python
import json, shutil, subprocess, tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

@dataclass
class Task:
    id: str
    prompt: str
    fixture: Path                    # trạng thái ban đầu của môi trường
    graders: list[Callable]          # mỗi grader: (workdir, transcript) -> bool

def tests_pass(workdir: Path, transcript: list) -> bool:
    """Grader bằng code: chấm outcome, không chấm lời agent nói."""
    r = subprocess.run(["pytest", "-q"], cwd=workdir, capture_output=True)
    return r.returncode == 0

def no_edit_outside(allowed: str) -> Callable:
    """Grader ràng buộc: agent không được sửa file ngoài thư mục cho phép."""
    def grade(workdir: Path, transcript: list) -> bool:
        edits = [s["path"] for s in transcript if s.get("tool") == "edit_file"]
        return all(p.startswith(allowed) for p in edits)
    return grade

def run_trial(task: Task, run_agent, out: Path, i: int) -> bool:
    with tempfile.TemporaryDirectory() as tmp:       # mỗi trial một môi trường sạch
        workdir = Path(tmp) / "work"
        shutil.copytree(task.fixture, workdir)
        transcript = run_agent(task.prompt, workdir)
        results = {g.__name__: g(workdir, transcript) for g in task.graders}
    passed = all(results.values())
    # Lưu transcript: thứ bạn sẽ đọc khi điểm trông lạ
    (out / f"{task.id}-{i}.json").write_text(
        json.dumps({"passed": passed, "graders": results, "transcript": transcript},
                   ensure_ascii=False, indent=2))
    return passed

def run_suite(tasks: list[Task], run_agent, k: int = 5, out=Path("runs")):
    out.mkdir(exist_ok=True)
    report = {}
    for t in tasks:
        passes = [run_trial(t, run_agent, out, i) for i in range(k)]
        report[t.id] = sum(passes) / k
    n = len(report)
    return {
        "ti_le_pass_trung_binh": sum(report.values()) / n,
        f"pass@{k}": sum(r > 0 for r in report.values()) / n,     # ít nhất 1 lần đúng
        f"pass^{k}": sum(r == 1 for r in report.values()) / n,    # cả k lần đều đúng
        "task_chap_chon": [t for t, r in report.items() if 0 < r < 1],
    }
```

Mấy lựa chọn thiết kế đáng chú ý:

- Mỗi trial copy fixture vào thư mục tạm, không trial nào thấy dấu vết của trial khác.
- Grader chính chấm outcome (`workdir`). Grader đọc transcript chỉ dùng cho ràng buộc.
- Danh sách `task_chap_chon` (pass lúc được lúc không) thường là chỗ đáng đọc nhất: hoặc task mơ hồ, hoặc agent có một failure mode xác suất.

Cách tính pass@k và pass^k ở trên là ước lượng thô theo từng task với đúng k trial. Nếu bạn chạy n trial nhiều hơn k rồi muốn ước lượng pass^k, cần công thức tổ hợp như trong paper τ-bench.

## Đọc transcript trước khi viết metric

Đây là phần mình thấy giá trị nhất và cũng ngược với thói quen kỹ sư nhất. Hamel Husain và Shreya Shankar gọi error analysis là hoạt động quan trọng nhất trong eval, và nói họ dành 60-80% thời gian phát triển cho error analysis và đánh giá. Lập luận: bạn không biết hệ thống hỏng kiểu gì cho tới khi nhìn nó hỏng. Viết metric trước là đo những lỗi bạn tưởng tượng ra.

Quy trình họ mô tả mượn từ nghiên cứu định tính:

1. **Open coding.** Lấy trace thật (hoặc sinh từ đầu vào tổng hợp nếu chưa có người dùng), đọc từng cái, ghi chú tự do điều gì sai. Chưa phân loại vội. Họ khuyên tự tay chú thích ít nhất 30 trace trước khi nhờ model gợi ý, và xem ít nhất 100 trace.
2. **Axial coding.** Gom các ghi chú thành một bảng phân loại failure mode. Họ coi đây là bước quan trọng nhất.
3. **Dừng khi bão hòa.** Khi các trace mới không còn lộ ra failure mode mới hay làm đổi nhóm cũ nữa.
4. Đếm tần suất từng nhóm, sửa nhóm lớn nhất trước. Chỉ viết evaluator tự động cho nhóm nào vẫn còn sau khi sửa.

Từ đó ra mấy quan điểm khá gắt của họ:

- **Metric chung chung có sẵn** (helpfulness, coherence, hay độ tương đồng kiểu ROUGE, BERTScore) tốn thời gian và tạo tự tin giả khi dùng làm thước đo chất lượng. Cùng lắm dùng để lọc ra trace đáng xem.
- **Chấm nhị phân pass/fail thay vì thang 1-5.** Thang điểm cho phép né quyết định ("cho 3 đi"), và không ai biết phải làm gì với một điểm 3 hay 4. Muốn đo tiến bộ dần dần thì tách thành nhiều tiêu chí con, mỗi tiêu chí vẫn nhị phân.
- **Dashboard ít hữu ích** hơn một bản tường thuật: cái gì đổi, nguyên nhân gốc, bước tiếp theo.

Anthropic đến cùng kết luận từ phía khác: đừng tin điểm số cho tới khi có người đã kiểm tra grader và đọc transcript, vì đọc transcript là cách duy nhất phân biệt agent sai thật với grader từ chối một lời giải hợp lệ.

### Chỗ các nguồn bất đồng: viết eval trước hay sau?

OpenAI khuyên "eval-driven development": đánh giá sớm và thường xuyên, viết test có phạm vi rõ ở mọi giai đoạn. Anthropic cũng mô tả việc viết eval cho năng lực trước khi agent làm được. Hamel Husain trả lời câu hỏi này là "nhìn chung là không": viết evaluator trước tính năng nghe hấp dẫn nhưng tạo nhiều vấn đề hơn nó giải quyết, vì bạn đo lỗi tưởng tượng thay vì lỗi phát hiện được. Ngoại lệ ông chấp nhận là ràng buộc tường minh, tiêu chí rõ từ trước, kiểu "không bao giờ nhắc tên đối thủ".

Mình nghĩ khác biệt đến từ loại sản phẩm. Với agent có outcome kiểm được bằng máy (code pass test, DB đúng trạng thái), task viết trước chính là đặc tả, và cách của Anthropic hợp lý. Với đầu ra mở và chủ quan (trợ lý trả lời khách, tóm tắt), bạn không viết được tiêu chí tốt cho tới khi thấy đầu ra thật hỏng thế nào, và cách của Hamel đỡ lãng phí hơn. Đây là suy luận của mình, không nguồn nào nói vậy.

## Hiệu chỉnh LLM judge với nhãn người

Một LLM judge chưa hiệu chỉnh là một nguồn nhiễu có vẻ ngoài của một con số. Quy trình mà Hamel Husain gọi là "critique shadowing":

1. Chọn một (tối đa hai) người có phán đoán nghiệp vụ mà sản phẩm phụ thuộc vào. Lỗi thường gặp là kỹ sư tự đóng vai này.
2. Người đó chấm pass/fail từng mẫu và viết lời phê giải thích, đủ chi tiết để dùng làm ví dụ few-shot.
3. Viết prompt judge, dùng các lời phê làm ví dụ, lặp cho tới khi judge khớp với người.
4. Đo độ khớp trên tập giữ riêng, chưa từng dùng khi viết prompt.

Về đo độ khớp, đừng dùng tỉ lệ đồng ý thô. Ví dụ của Hamel: nếu chỉ 5% đầu ra là lỗi, một judge luôn trả "pass" đạt 95% đồng ý mà không bắt được lỗi nào. Báo cáo riêng hai số: true positive rate (bao nhiêu phần trăm ca người chấm fail được judge chấm fail) và true negative rate. Ông gợi ý chia dữ liệu có nhãn thành ba phần: 10-20% làm ví dụ trong prompt, 40-45% để tinh chỉnh, 40-45% giữ lại cho một lần kiểm cuối.

Một rubric judge mẫu cho tiêu chí "câu trả lời bám vào tài liệu":

```text
Bạn chấm MỘT tiêu chí duy nhất: mọi khẳng định mang tính dữ kiện trong CÂU TRẢ LỜI
có được TÀI LIỆU bên dưới chống đỡ hay không.

Cách làm:
1. Liệt kê từng khẳng định dữ kiện trong câu trả lời (con số, ngày, chính sách, tên).
2. Với mỗi khẳng định, trích nguyên văn đoạn tài liệu chống đỡ nó, hoặc ghi "không tìm thấy".
3. Kết luận.

PASS: mọi khẳng định đều có đoạn trích chống đỡ.
FAIL: có ít nhất một khẳng định không có đoạn trích, hoặc mâu thuẫn với tài liệu.
UNKNOWN: tài liệu bị thiếu hoặc cắt cụt nên không thể kiểm.

Không chấm giọng văn, độ dài, hay mức độ hữu ích. Những thứ đó có grader khác.

Ví dụ đã được người chấm:
[2-4 cặp (câu trả lời, tài liệu, lời phê, nhãn) lấy từ tập ví dụ]

TÀI LIỆU:
{tai_lieu}

CÂU TRẢ LỜI:
{cau_tra_loi}

Trả về JSON: {"khang_dinh": [...], "ly_do": "...", "nhan": "PASS" | "FAIL" | "UNKNOWN"}
```

Các lựa chọn trong rubric đều có lý do từ nguồn. Mỗi judge một tiêu chí: cả Anthropic (judge tách biệt cho từng chiều) lẫn Hamel (mỗi failure mode một judge, chỉ đưa cho judge phần trace nó cần) đều khuyên vậy. Có lối thoát "UNKNOWN": Anthropic khuyên cho judge một cách nói không biết để nó khỏi bịa phán quyết. Lý do đặt trước nhãn để model lập luận trước khi kết luận. Nhãn nhị phân theo lập luận ở phần trên.

Có nên dùng cùng model với agent để làm judge không? Hamel cho rằng thường là ổn, miễn độ khớp với nhãn người tốt. Điều kiện sau mới là thứ quyết định.

## Bẫy thường gặp

**Eval bão hòa.** Khi agent pass hết các task giải được, bộ eval không còn cho thấy tiến bộ. Anthropic lấy ví dụ SWE-bench Verified: điểm đi từ khoảng 30% lên trên 80%, và giờ cải thiện chỉ hiện ra dưới dạng chênh lệch nhỏ. Hamel nói tương tự: evaluator pass mọi ví dụ thì không cho thông tin mới. Trước khi bỏ, hãy thử làm nó khó hơn. Với regression suite thì gần 100% là trạng thái mong muốn, đừng nhầm hai trường hợp.

**Grader sai.** Trường hợp CORE-Bench trong bài của Anthropic: Opus 4.5 ban đầu đạt 42%. Khi soát lại, họ thấy grader từ chối "96.12" vì đáp án kỳ vọng là "96.124991…", cùng với đặc tả task mơ hồ và các task ngẫu nhiên không tái lập được. Sau khi sửa và dùng scaffold ít gò bó hơn, điểm lên 95%. Điểm thấp bất thường là tín hiệu để đọc transcript trước khi kết luận gì về model.

**Task mơ hồ hoặc lệch với grader.** Cũng từ bài đó: có benchmark yêu cầu agent "đạt" một ngưỡng điểm nhưng grader lại đòi "vượt" ngưỡng, thành ra phạt đúng model làm theo chỉ dẫn. Và trong τ2-bench, Opus 4.5 bị chấm "fail" một task đặt vé vì tìm ra kẽ hở trong chính sách, cho người dùng một phương án tốt hơn lời giải tham chiếu. Grader kiểm tra đường đi cứng sẽ gặp chuyện này thường xuyên hơn khi model giỏi lên.

**Grader bị lách.** Nếu grader chỉ kiểm "file output tồn tại", agent sẽ học cách tạo file rỗng. Grader phải đòi hỏi giải được bài toán thật.

**Overfit vào bộ eval.** Sửa prompt 30 vòng trên cùng 30 task thì prompt sẽ thuộc bài. Mấy cách giảm: giữ một tập task không bao giờ dùng khi chỉnh, bổ sung task mới từ lỗi production đều đặn, và nghi ngờ mọi cải thiện mà bạn không giải thích được bằng cơ chế. Điều này mình suy từ nguyên tắc chia train/test thông thường; các nguồn trên nói kỹ về chia dữ liệu cho judge hơn là cho bộ task.

**Eval không giống production.** Agent trong eval phải chạy gần giống agent thật: cùng harness, cùng tool, cùng giới hạn. OpenAI xếp bộ dữ liệu eval không tái hiện đúng lưu lượng thật vào nhóm anti-pattern.

## Điều còn mở, chưa chắc

- **Viết eval trước hay sau khi thấy lỗi.** Như đã nói, OpenAI và Anthropic nghiêng về viết sớm, Hamel Husain nghiêng về viết sau error analysis. Chưa có dữ liệu so sánh hai cách, chỉ có kinh nghiệm của từng bên.
- **Điểm từng phần hay nhị phân.** Anthropic gợi ý partial credit cho task nhiều bước, Hamel muốn mọi thứ nhị phân và tách tiêu chí con. Hai cách có thể quy về nhau, nhưng cách tổng hợp thành một con số để ra quyết định thì chưa ai có công thức thuyết phục.
- **LLM judge tin được đến đâu khi model agent đổi.** Judge hiệu chỉnh trên đầu ra của model cũ có còn khớp nhãn người trên đầu ra của model mới không? Mình chưa thấy nguồn nào trả lời có số liệu. Cách an toàn là chấm tay lại một mẫu sau mỗi lần đổi model.
- **Bao nhiêu trial là đủ.** Các nguồn mình đọc không đưa ra con số chung. Nó phụ thuộc độ dao động của task và mức chênh lệch bạn cần phát hiện, và chi phí mỗi trial của agent chạy dài thường là ràng buộc thật.
- **Phân bổ công sức giữa các lớp.** Anthropic ví với mô hình "phô mai Thụy Sĩ": eval tự động, giám sát production, A/B test, phản hồi người dùng, đọc transcript thủ công, mỗi lớp có lỗ riêng. Chia công sức thế nào thì chưa có công thức.

## Nguồn đọc thêm

- [Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents) (Anthropic). Nguồn của bộ thuật ngữ, bảng ba loại grader và các ca grader sai dùng trong bài. Nên đọc đầu tiên nếu bạn làm agent.
- [AI Evals FAQ](https://hamel.dev/blog/posts/evals-faq/) (Hamel Husain, Shreya Shankar). Rất dài, quan điểm rõ: error analysis trước, nhị phân, không metric chung chung. Đọc để có đối trọng với cách tiếp cận "viết eval trước".
- [Creating a LLM-as-a-Judge That Drives Business Results](https://hamel.dev/blog/posts/llm-judge/) (Hamel Husain). Quy trình critique shadowing từng bước, kèm lý do không dùng thang 1-5.
- [Evaluation best practices](https://developers.openai.com/api/docs/guides/evaluation-best-practices) (OpenAI). Ngắn, có danh sách anti-pattern và các loại eval riêng cho agent đơn, đa agent.
- [τ-bench: A Benchmark for Tool-Agent-User Interaction in Real-World Domains](https://arxiv.org/abs/2406.12045) (Yao và cộng sự, 2024). Nơi đưa ra pass^k, và là ví dụ mẫu về chấm bằng trạng thái DB cuối.
- [SWE-bench: Can Language Models Resolve Real-World GitHub Issues?](https://arxiv.org/abs/2310.06770) (Jimenez và cộng sự, 2023). Ví dụ kinh điển về lấy task từ lỗi thật và chấm bằng test.
