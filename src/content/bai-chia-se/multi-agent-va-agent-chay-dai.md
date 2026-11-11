---
title: "Multi-agent và agent chạy dài: khi nào chia việc, bàn giao thế nào"
date: 2026-11-10
tags: [Agent, Harness]
description: Các pattern multi-agent, cái giá phải trả, chỗ Cognition và Anthropic thực ra đồng ý với nhau, và cách dựng harness để agent làm việc qua nhiều context window. Kèm bảng quyết định một hay nhiều agent và mẫu brief bàn giao.
loai: chuyen-sau
---

Giữa tháng 6/2025 có hai bài viết ra cách nhau một ngày với kết luận trông như ngược nhau. Cognition (đội làm Devin) viết "Don't Build Multi-Agents" ngày 12/6. Anthropic viết "How we built our multi-agent research system" ngày 13/6, kể rằng hệ nhiều agent của họ vượt xa một agent đơn. Đọc kỹ thì hai bài nói về hai loại việc khác nhau.

Bài này đi qua các pattern multi-agent, cái giá của chúng, cuộc tranh luận đó, rồi sang một bài toán họ hàng: làm sao để agent làm một việc kéo dài nhiều giờ, qua nhiều context window (phần văn bản model nhìn thấy trong một lượt gọi), mà không quên mình đang làm gì. Theo mình, hai chủ đề nối với nhau ở chỗ bàn giao: giữa hai agent hay giữa hai phiên của cùng một agent thì bên nhận cũng chỉ biết những gì được viết ra.

Quy ước: câu có tên nguồn là nguồn nói. Chỗ nào mình viết "theo mình", "mình đọc", "mình thêm" là suy luận của người viết. Sub-agent là gì và nằm ở đâu trong harness xem ở bài [Giải phẫu một agent harness](/bai-chia-se/giai-phau-agent-harness/).

## Vì sao lại muốn nhiều agent

Mình tách ba lý do kỹ thuật, vì mỗi lý do dẫn tới một thiết kế khác:

1. Cô lập context. Một tác vụ tìm kiếm có thể đốt hàng chục nghìn token cho kết quả thô mà agent chính chỉ cần một đoạn kết luận. Giao nó cho sub-agent thì phần thô nằm lại ở context của sub-agent.
2. Song song. Các nhánh việc độc lập chạy cùng lúc thì nhanh hơn chạy nối tiếp.
3. Góc nhìn độc lập. Anthropic ghi nhận agent chấm bài của chính nó thì lệch về phía khen; một agent khác với context sạch dễ chỉnh cho khắt khe hơn.

"Mỗi agent một vai như trong công ty" (agent PM, agent dev, agent tester nói chuyện với nhau) không nằm trong danh sách này. Theo mình, nếu việc chia vai không mua được một trong ba thứ trên, nó chỉ thêm chỗ để thông tin rơi rụng.

## Bốn pattern thường gặp

```text
(1) Orchestrator-worker              (2) Sub-agent cô lập context

      orchestrator                        agent chính (context gọn)
      /    |    \                             |  brief        ^
  worker worker worker                        v               | tóm tắt 1-2k token
      \    |    /                         sub-agent (đọc 50k token rồi bỏ)
       tổng hợp

(3) Planner / generator / evaluator  (4) Song song hóa

  planner -> spec                         cùng một việc x N  -> bỏ phiếu
               |                          N phần độc lập     -> ghép lại
           generator <--- phản hồi ---+
               |                      |
            sản phẩm ---> evaluator --+
```

Định nghĩa gốc của các mẫu orchestrator-workers, parallelization và evaluator-optimizer nằm trong "Building effective agents", đã tóm ở bảng năm mẫu workflow của bài [Giải phẫu một agent harness](/bai-chia-se/giai-phau-agent-harness/). Ở đây mình chỉ ghi phần cần cho các mục sau.

**Orchestrator-worker.** Một agent điều phối chia việc theo đầu vào, giao cho các worker, rồi tổng hợp. Hệ thống Research của Anthropic là ví dụ: agent dẫn đầu lập kế hoạch, sinh 3-5 sub-agent tìm kiếm song song, nhận kết quả, quyết định có cần tìm thêm không, cuối cùng một agent riêng gắn trích dẫn.

**Sub-agent cô lập context.** Agent chính vẫn là nơi duy nhất ra quyết định và ghi kết quả, sub-agent chỉ đi trả lời một câu hỏi rồi về. Bài về context engineering của Anthropic mô tả sub-agent có thể dùng hàng chục nghìn token để khám phá nhưng chỉ trả về bản tóm tắt, thường 1.000-2.000 token. Theo mình đây là dạng ít rủi ro nhất, vì sub-agent không ghi gì.

**Planner, generator, evaluator.** Tách bên lập đặc tả, bên làm và bên chấm. Ví dụ cụ thể ở phần agent chạy dài bên dưới.

**Song song hóa.** Chia một việc thành các phần độc lập chạy cùng lúc, hoặc chạy cùng một việc nhiều lần lấy nhiều góc nhìn.

## Cái giá

### Token

Anthropic đưa con số từ dữ liệu của chính họ: agent dùng khoảng 4 lần token so với chat, còn hệ multi-agent khoảng 15 lần. Họ cũng phân tích rằng trên bộ đánh giá BrowseComp, riêng lượng token đã dùng giải thích 80% phương sai kết quả; cộng thêm số lượt gọi tool và lựa chọn model thì giải thích được 95%.

Anthropic đọc con số này theo hướng: multi-agent là cách tiêu được nhiều token hơn cho một bài toán, vì mỗi sub-agent có context window riêng, và chỉ đáng khi giá trị của tác vụ đủ lớn. Mình thêm một cách đọc thận trọng hơn: một phần lợi ích đến từ việc tiêu nhiều hơn chứ không phải từ kiến trúc, và bài không có phép so sánh nào khống chế cùng ngân sách token.

### Lỗi phối hợp

Ví dụ của Cognition: giao việc "làm bản sao Flappy Bird" cho hai sub-agent, một làm nền và ống, một làm con chim. Sub-agent thứ nhất dựng nền kiểu Super Mario, sub-agent thứ hai làm con chim không khớp cả hình lẫn cách di chuyển. Agent tổng hợp nhận về hai mảnh không ráp được. Theo Cognition, truyền nguyên văn yêu cầu gốc cho cả hai cũng không đủ, vì trong hệ thật agent chính đã qua nhiều lượt trao đổi và tool call trước khi chia việc.

Cognition rút ra hai nguyên tắc: chia sẻ context đầy đủ (cả vết hành động chứ không chỉ từng tin nhắn), và mỗi hành động đều mang quyết định ngầm, quyết định ngầm mâu thuẫn nhau thì kết quả hỏng.

Phía Anthropic cũng ghi lại các lỗi phối hợp ở phiên bản đầu: sinh 50 sub-agent cho câu hỏi đơn giản, sub-agent lùng mãi những nguồn không tồn tại, hoặc làm phiền nhau bằng quá nhiều cập nhật.

### Mất ngữ cảnh khi bàn giao

Đoạn này là suy luận của mình từ hai mục trên. Bản tóm tắt 1.000-2.000 token là nén có mất mát: sub-agent biết vì sao nó loại một hướng, đã thử gì và hỏng thế nào; agent chính chỉ thấy kết luận. Chiều ngược lại cũng vậy: brief giao việc là tất cả những gì sub-agent biết về mục tiêu. Mỗi mũi tên trong sơ đồ ở trên là một chỗ thông tin có thể rơi.

### Vận hành

Anthropic kể hệ nhiều agent khó debug vì không tất định, nên họ cần tracing đầy đủ ở production, checkpoint để chạy tiếp từ chỗ lỗi thay vì chạy lại từ đầu, và triển khai kiểu rainbow (dịch dần lưu lượng, giữ song song bản cũ và mới) để không cắt ngang agent đang chạy dở. Agent dẫn đầu của họ còn chạy đồng bộ, chờ cả lứa sub-agent xong mới đi tiếp, nên một sub-agent chậm chặn cả hệ.

## Cognition và Anthropic: bất đồng ở đâu, đồng ý ở đâu

| | Cognition (06/2025) | Anthropic (06/2025) |
| --- | --- | --- |
| Loại việc | Viết code: nhiều quyết định phụ thuộc lẫn nhau | Nghiên cứu: tìm và đọc theo nhiều hướng độc lập |
| Sub-agent làm gì | Ghi (tạo ra các phần của sản phẩm) | Đọc (tìm, lọc, tóm tắt) |
| Kết luận | Mặc định một agent tuyến tính; việc dài thì thêm một model nén lịch sử | Orchestrator-worker cho việc theo chiều rộng |
| Dè dặt | Viết rằng lý thuyết của họ nhiều khả năng chưa hoàn hảo | Viết rằng multi-agent chưa hợp với việc cần chung context, và đa số việc code ít phần song song hóa được |

Anthropic báo cáo hệ gồm Opus 4 dẫn đầu và các sub-agent Sonnet 4 vượt Opus 4 chạy đơn 90,2% trên bộ đánh giá nghiên cứu nội bộ. Con số do họ tự đo, trên bộ đánh giá không công khai, cho riêng loại việc nghiên cứu. Cognition thì ghi nhận Claude Code thời điểm ấy có sub-agent, nhưng sub-agent không làm song song với agent chính và thường chỉ trả lời một câu hỏi xác định chứ không viết code; họ coi đó là thiết kế đơn giản có chủ đích.

Mình đọc ra ba điểm hai bên thực ra đồng ý:

1. Việc đọc thì chia được, việc ghi thì không nên. Tìm kiếm song song ít rủi ro vì các kết quả không cần khớp nhau từng chi tiết; viết song song thì mỗi bên tự chọn phong cách, pattern, cách xử lý trường hợp biên, và các lựa chọn đó đụng nhau.
2. Context là nút thắt. Anthropic dành nhiều chỗ cho cách viết brief cho sub-agent; Cognition nói cùng điều bằng ngôn ngữ "chia sẻ context đầy đủ".
3. Kiến trúc không chuộc được việc bàn giao tệ.

Tháng 4/2026, tác giả bài của Cognition (Walden Yan) viết tiếp "Multi-Agents: What's Actually Working". Câu chốt của bài: hệ multi-agent hiện hoạt động tốt nhất khi việc ghi vẫn đơn luồng, còn các agent bổ sung đóng góp trí tuệ chứ không đóng góp hành động. Ba dạng họ mô tả:

- **Agent review với context sạch.** Họ báo cáo Devin Review trung bình bắt được 2 bug mỗi PR, khoảng 58% trong đó nghiêm trọng.
- **"Smart friend".** Model chính nhỏ hơn, tự quyết khi nào gọi một model mạnh hơn như một tool. Cách chia sẻ context họ thấy đủ tốt là đưa cho model mạnh một bản fork toàn bộ context. Chỗ chưa được, theo chính họ: cách này chạy tốt khi model chính là model hàng đầu, không tốt khi model chính yếu hơn.
- **Manager và các agent con.** Một agent chia việc lớn, sinh agent con và tổng hợp. Lỗi họ gặp: manager chỉ đạo quá chi tiết trong khi thiếu hiểu biết sâu về codebase, agent tưởng nhầm rằng nó chung trạng thái với agent con, và agent con mặc định không chủ động báo lại cho manager. Họ gọi mạng lưới agent tự thương lượng không cấu trúc là thứ gây xao nhãng.

Mình đọc diễn biến này như sau: tranh luận "một hay nhiều" đã chuyển thành câu hỏi "agent nào được quyền ghi, và thông tin đi giữa các agent bằng đường nào".

## Bảng quyết định: một agent hay nhiều agent

Bảng này là tổng hợp của mình từ các nguồn trên, chưa được kiểm bằng thí nghiệm nào.

| Dấu hiệu của tác vụ | Nên chọn | Vì sao |
| --- | --- | --- |
| Các bước phụ thuộc chặt, cùng sửa một sản phẩm (đa số việc code) | Một agent, ghi đơn luồng | Quyết định ngầm phải nhất quán |
| Cần khám phá nhiều (đọc codebase, tìm tài liệu) nhưng chỉ cần kết luận | Một agent chính + sub-agent chỉ đọc | Giữ context chính gọn; sub-agent không ghi nên không xung đột |
| Nhiều câu hỏi con độc lập, theo chiều rộng | Orchestrator-worker song song | Song song được thật, thông tin vượt một context window |
| Chất lượng khó tự đánh giá (giao diện, bug tinh vi) | Thêm evaluator hoặc reviewer với context sạch | Agent tự chấm thường dễ dãi |
| Việc kéo dài nhiều giờ, vượt một context window | Một agent qua nhiều phiên, trạng thái nằm ở file | Bài toán là trí nhớ, không phải số agent |
| Việc đơn giản, vài lượt gọi tool | Một agent, không thêm gì | Chi phí điều phối lớn hơn lợi ích |
| Giá trị mỗi lần chạy thấp, khối lượng lớn | Một agent hoặc workflow cố định | Mức token gấp nhiều lần khó hoàn vốn |

Một quy tắc phụ lấy từ prompt của hệ Research: cho agent điều phối một thang công sức tường minh. Tra một dữ kiện đơn giản thì 1 agent với 3-10 lượt gọi tool; so sánh trực tiếp thì 2-4 sub-agent, mỗi cái 10-15 lượt; nghiên cứu phức tạp mới dùng hơn 10 sub-agent. Anthropic thêm thang này vì agent hay huy động quá tay với câu hỏi đơn giản.

## Agent chạy dài: bài toán là trí nhớ giữa các phiên

Một tác vụ vài giờ sẽ vượt context window. Có ba cách xử lý, thường dùng lẫn nhau: compaction (tóm tắt lịch sử rồi chạy tiếp), ghi chú có cấu trúc (agent tự viết trạng thái ra file), và context reset (bỏ hẳn context, mở phiên mới chỉ với những gì đã ghi ra đĩa). Hai cách đầu đã giải thích ở bài [Context engineering cho agent](/bai-chia-se/context-engineering-cho-agent/); phần này nói về cách thứ ba.

Bài "Effective harnesses for long-running agents" (Anthropic, 11/2025) mô tả context reset cho bài toán dựng một ứng dụng web hoàn chỉnh. Họ viết rằng compaction một mình không đủ: đưa cho Opus 4.5 chạy trong Claude Agent SDK chỉ một prompt cấp cao rồi để nó chạy qua nhiều context window thì vẫn không ra ứng dụng đạt chất lượng production. Ba kiểu hỏng lặp lại:

1. Làm tất cả một lèo. Agent cố dựng cả ứng dụng trong một phiên, hết context giữa chừng, để lại tính năng dở dang không ghi chép.
2. Tuyên bố xong sớm. Phiên sau thấy đã có nhiều thứ, kết luận là việc đã hoàn thành.
3. Đánh dấu xong mà chưa kiểm đầu-cuối.

Hình dung của họ là một nhóm kỹ sư làm theo ca, mỗi ca vào không nhớ gì về ca trước. Giải pháp là chuẩn hóa những gì ca trước để lại.

### Initializer và coding agent

Cùng một harness (cùng system prompt, cùng tool), chỉ khác prompt đầu tiên. Phiên đầu là initializer, dựng môi trường. Mọi phiên sau là coding agent, mỗi phiên làm một tính năng rồi để lại trạng thái sạch.

| Thứ để lại | Ai tạo | Dùng để làm gì |
| --- | --- | --- |
| Danh sách tính năng (JSON) | Initializer | Định nghĩa "xong" cho cả dự án, chống tuyên bố xong sớm |
| File tiến độ | Initializer tạo, mỗi phiên cập nhật | Phiên sau biết phiên trước làm gì, vướng gì |
| Lịch sử git | Mỗi phiên commit | Checkpoint: quay lui khi hỏng, đọc log để hiểu diễn biến |
| `init.sh` | Initializer | Khởi động môi trường mà không phải mò lại cách chạy |

Danh sách tính năng trong ví dụ dựng bản sao claude.ai của họ có hơn 200 mục. Mỗi mục có dạng sau (mình dịch nội dung sang tiếng Việt):

```json
{
  "category": "functional",
  "description": "Nút New chat tạo một cuộc trò chuyện mới",
  "steps": [
    "Mở giao diện chính",
    "Bấm nút New chat",
    "Kiểm tra một cuộc trò chuyện mới được tạo",
    "Kiểm tra khung chat hiển thị trạng thái chào mừng",
    "Kiểm tra cuộc trò chuyện xuất hiện ở thanh bên"
  ],
  "passes": false
}
```

Hai chi tiết thiết kế. Mọi mục bắt đầu với `passes: false`, và coding agent chỉ được sửa đúng trường này; prompt viết rõ việc xóa hay sửa test là không chấp nhận được. Họ chọn JSON thay vì Markdown vì thấy model ít sửa bậy hoặc ghi đè file JSON hơn.

Mỗi phiên coding agent mở đầu theo cùng một trình tự. Pseudo-code dưới là mình dựng lại từ mô tả; bài gốc chỉ nêu tên `claude-progress.txt` và `init.sh`, còn `feature_list.json` là tên mình đặt:

```python
def coding_session():
    run("pwd")                               # xác nhận thư mục làm việc
    read("claude-progress.txt")              # phiên trước làm gì
    run("git log --oneline -20")             # diễn biến gần đây
    features = read_json("feature_list.json")

    run("./init.sh")                         # dựng môi trường, chạy dev server
    smoke_test()                             # thử luồng cơ bản TRƯỚC khi làm việc mới

    f = next_unfinished(features)            # chọn một tính năng chưa đạt, ưu tiên cao nhất
    implement(f)
    if verify_end_to_end(f["steps"]):        # thao tác trên trình duyệt như người dùng
        f["passes"] = True
        write_json("feature_list.json", features)
    run("git commit")                        # message mô tả rõ
    append("claude-progress.txt", summary())
```

Bước smoke test trước khi làm việc mới bắt lỗi do phiên trước để lại. Về kiểm chứng, họ ghi nhận model kiểm tra kém cho tới khi được yêu cầu rõ dùng công cụ tự động hóa trình duyệt (Puppeteer MCP) và thử như người dùng; lúc đó nó bắt được những bug không thấy khi chỉ đọc code. Giới hạn họ nêu: agent không thấy hộp thoại alert gốc của trình duyệt qua công cụ này, nên các tính năng dựa vào đó lỗi nhiều hơn.

Theo mình, điểm chung của bốn thứ trong bảng là trạng thái nằm ở hệ thống file và git, không nằm trong context.

### Bài tiếp theo: tách bên làm và bên chấm

Tháng 3/2026, Anthropic viết tiếp "Harness design for long-running application development" với hai quan sát. Thứ nhất là "context anxiety": một số model vội kết thúc công việc khi nghĩ mình sắp hết context; họ nêu Sonnet 4.5 bị đủ nặng để compaction không cứu được. Thứ hai, agent chấm bài của chính mình thì lệch về phía khen, nhất là với việc chủ quan như thiết kế giao diện.

Kiến trúc họ dùng là ba agent. Planner mở rộng một prompt 1-4 câu thành đặc tả sản phẩm, tập trung vào bối cảnh sản phẩm và thiết kế kỹ thuật ở mức cao. Generator làm theo sprint. Evaluator dùng Playwright MCP bấm thử ứng dụng đang chạy rồi chấm. Trước mỗi sprint, generator đề xuất sẽ làm gì và kiểm bằng cách nào, evaluator duyệt, hai bên lặp tới khi thống nhất; họ gọi đó là "sprint contract".

Lý do tách evaluator: nó vẫn là LLM và vẫn có xu hướng dễ dãi, nhưng theo họ chỉnh một evaluator riêng cho khắt khe khả thi hơn nhiều so với bắt generator tự phê bình. Việc chỉnh cũng tốn công: ban đầu evaluator tìm ra lỗi thật rồi tự thuyết phục mình cho qua, và tác giả phải đọc log của nó, sửa prompt, lặp lại nhiều vòng. Cách hiệu chỉnh một model chấm với nhãn của người ở bài [Eval cho ứng dụng LLM và agent](/bai-chia-se/eval-cho-agent/).

Chi phí họ công bố cho cùng một đề bài (trình tạo game retro 2D, bản harness đầu dùng Opus 4.5): một agent đơn mất 20 phút và 9 USD, harness đầy đủ mất 6 giờ và 200 USD. Đắt hơn hơn 20 lần; đổi lại, theo họ, bản của agent đơn có tính năng cốt lõi không chạy được. Đây là một lần chạy trên một đề bài, không phải benchmark.

Phần mình thấy giá trị nhất là thái độ với harness: mỗi thành phần là một giả định về điều model chưa tự làm được, và giả định có hạn dùng. Với Opus 4.5 họ bỏ context reset vì model gần như hết context anxiety, các agent chạy một phiên liên tục có compaction tự động. Với Opus 4.6 họ bỏ cấu trúc sprint, evaluator chỉ chấm một lượt ở cuối. Planner được giữ vì thiếu nó generator làm thiếu phạm vi. Bài năm 2025 cần reset giữa các phiên, bài năm 2026 không cần nữa, và cả hai đều đúng với model của thời điểm đó.

## Viết brief cho sub-agent

Bài của Anthropic kể rằng brief ngắn kiểu "nghiên cứu tình trạng thiếu chip" khiến một sub-agent đi tìm khủng hoảng chip ô tô năm 2021 trong khi hai sub-agent khác làm trùng nhau về chuỗi cung ứng hiện tại. Theo họ mỗi brief cần bốn thứ: mục tiêu, định dạng đầu ra, hướng dẫn về tool và nguồn nên dùng, ranh giới nhiệm vụ. Mình thêm hai mục rút từ bài của Cognition: bối cảnh đã quyết (để sub-agent không ra quyết định ngầm trái với phần còn lại) và điều kiện dừng.

Mẫu dưới là của mình, dùng được cho cả sub-agent lẫn ghi chú bàn giao giữa hai phiên:

```text
MỤC TIÊU
[Một câu: cần biết hoặc cần có gì. Kèm lý do: kết quả sẽ được dùng để làm gì.]

BỐI CẢNH ĐÃ QUYẾT
[Các quyết định đã chốt mà bạn phải theo: stack, quy ước đặt tên, hướng đã loại và vì sao.]
[Những gì đã thử và kết quả, để không làm lại.]

PHẠM VI
Làm: [liệt kê cụ thể]
Không làm: [việc thuộc agent khác; file không được sửa; quyền ghi: có/không]

TOOL VÀ NGUỒN
[Tool nên dùng trước. Nguồn ưu tiên. Nguồn cần tránh.]

MỨC CÔNG SỨC VÀ ĐIỀU KIỆN DỪNG
[Ước lượng số lượt gọi tool. Dừng khi: đã đủ X, hoặc sau N lượt không có gì mới.]
[Nếu bị chặn hoặc phát hiện điều ảnh hưởng tới phần việc khác: dừng và báo, đừng tự xử lý.]

ĐỊNH DẠNG TRẢ VỀ
1. Kết luận (3-5 dòng)
2. Bằng chứng: đường dẫn file hoặc link, kèm trích đoạn ngắn
3. Điều chưa chắc và điều chưa kiểm
4. Việc đề xuất làm tiếp
```

Vài ghi chú khi dùng, đều là ý của mình trừ chỗ ghi nguồn:

- Mục "Bối cảnh đã quyết" là nơi ví dụ Flappy Bird hỏng. Nếu mục này dài quá một trang, có thể việc đó chưa nên tách ra.
- Yêu cầu "điều chưa chắc và điều chưa kiểm" buộc sub-agent tách kết luận đã kiểm chứng khỏi phỏng đoán, giảm mất mát khi nén.
- Giao kết quả mong muốn, đừng giao từng bước. Manager chỉ đạo quá chi tiết là lỗi Cognition ghi nhận ở chính hệ thống của họ.
- Dòng "dừng và báo" có vì agent con mặc định không tự báo lại, cũng theo Cognition.
- Với kết quả lớn, cho sub-agent ghi ra file rồi trả về đường dẫn kèm tóm tắt, thay vì chuyển toàn bộ nội dung qua agent điều phối.
- Brief cho reviewer là ngoại lệ có chủ ý: đưa diff và tiêu chí, không đưa lập luận của bên viết, để giữ góc nhìn độc lập.

## Điều còn mở / chưa chắc

- Agent chuyên biệt có hơn một agent đa năng cho việc code dài không? Bài tháng 11/2025 của Anthropic để ngỏ câu này; bài tháng 3/2026 cho rằng evaluator chỉ đáng tiền với việc vượt quá khả năng model tự làm ổn định, và bằng chứng là vài lần chạy trên vài đề bài.
- Viết song song có bao giờ ổn không? Các demo quy mô lớn mà Cognition nhắc tới (trình duyệt web, trình biên dịch C) có điểm chung là tiêu chí thành công đơn giản và kiểm chứng tự động được. Theo chính họ, phần mềm thông thường hiếm khi có điều đó.
- Bao nhiêu phần lợi ích của multi-agent đến từ kiến trúc, bao nhiêu từ việc tiêu nhiều token hơn? Phân tích phương sai của Anthropic gợi ý phần sau không nhỏ. Mình chưa thấy so sánh nào khống chế cùng ngân sách token.
- Harness nên dày tới đâu? Mỗi đời model lại làm vài thành phần thành thừa, nên đầu tư nặng vào điều phối có thể lỗi thời nhanh. Ngược lại, chờ model tự giỏi lên thì hôm nay không giao được việc.
- Các con số trong bài (90,2%, 15 lần token, 58% bug nghiêm trọng, 9 USD so với 200 USD) đều do bên làm sản phẩm tự công bố, trên dữ liệu và bộ đánh giá nội bộ. Chi tiết về sản phẩm (Claude Code, Devin) mô tả theo thời điểm bài gốc được viết và có thể đã đổi.

## Nguồn đọc thêm

- [How we built our multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system) (Anthropic, 06/2025): nguồn của các con số về token và phần viết brief cho sub-agent; đọc cả phần vận hành production.
- [Don't Build Multi-Agents](https://cognition.com/blog/dont-build-multi-agents) (Cognition, 06/2025): ví dụ Flappy Bird và hai nguyên tắc về chia sẻ context.
- [Multi-Agents: What's Actually Working](https://cognition.com/blog/multi-agents-working) (Cognition, 04/2026): bản cập nhật sau mười tháng, nêu rõ cái gì chạy được và cái gì chưa.
- [Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents) (Anthropic, 11/2025): initializer, coding agent, danh sách tính năng, bảng lỗi và cách xử lý.
- [Harness design for long-running application development](https://www.anthropic.com/engineering/harness-design-long-running-apps) (Anthropic, 03/2026): planner, generator, evaluator, chi phí thật và bài học gỡ dần harness.
- [Building effective agents](https://www.anthropic.com/engineering/building-effective-agents) (Anthropic, 12/2024): định nghĩa gốc của orchestrator-workers, parallelization, evaluator-optimizer.
- [Effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) (Anthropic, 09/2025): compaction, ghi chú có cấu trúc và sub-agent nhìn từ góc quản lý context.
