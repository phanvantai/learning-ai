---
title: "Giải phẫu một agent harness: vòng lặp, chín bộ phận và vì sao cùng model mà kết quả khác nhau"
date: 2026-11-08
tags: [Harness, Agent]
description: Bản đồ tổng thể về agent harness cho kỹ sư phần mềm, gồm vòng lặp tối thiểu khoảng 30 dòng, chín bộ phận bọc quanh model, ranh giới workflow và agent, và những lúc không nên dùng agent.
loai: chuyen-sau
---

Một coding agent như Claude Code, Codex hay Cursor gồm hai phần: model sinh ra token, và mọi thứ còn lại. "Mọi thứ còn lại" là harness. Tài liệu của Claude Code định nghĩa khá gọn: đó là lớp bọc quanh model, cung cấp tool và quản lý phần context mà model nhìn thấy ([How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works)).

Bài này là bản đồ tổng thể: vòng lặp lõi trông thế nào, harness thật có những bộ phận gì, mỗi bộ phận tồn tại để chặn kiểu hỏng nào. Riêng phần quản lý context mình chỉ lướt qua, vì nó đủ lớn để có bài riêng về context engineering.

## Vòng lặp tối thiểu

Lõi của mọi agent là một vòng `while`: gọi model, nếu model yêu cầu tool call (model trả về tên hàm và tham số thay vì văn bản) thì chạy tool, nối kết quả vào hội thoại, gọi lại. Hướng dẫn của OpenAI nói thẳng rằng khái niệm vòng lặp while này là trung tâm của cách agent vận hành, và mọi cách điều phối đều cần một khái niệm "run" chạy đến khi gặp điều kiện thoát ([A practical guide to building agents](https://cdn.openai.com/business-guides-and-resources/a-practical-guide-to-building-agents.pdf)).

Dưới đây là pseudo-code kiểu Python, không gắn với SDK nào:

```python
def run_agent(task, tools, system_prompt, max_turns=50):
    messages = [task_message(task)]
    for turn in range(max_turns):
        if count_tokens(system_prompt, tools, messages) > CONTEXT_BUDGET:
            messages = compact(messages)            # tóm tắt lịch sử cũ

        reply = call_model(system=system_prompt,
                           tools=[t.schema for t in tools.values()],
                           messages=messages)
        messages.append(reply)

        if not reply.tool_calls:                    # model không gọi tool nữa
            return reply.text                       # => coi như xong việc

        for call in reply.tool_calls:
            tool = tools.get(call.name)
            if tool is None:
                result = error(f"Không có tool tên {call.name}")
            elif not permitted(call):               # luật cứng hoặc hỏi người
                result = error("Bị từ chối: " + deny_reason(call))
            else:
                try:
                    result = tool.run(**call.args)  # chạy trong sandbox
                except Exception as e:
                    result = error(repr(e))         # lỗi cũng là thông tin
            messages.append(tool_result(call.id, truncate(result)))

    return give_up(messages, reason="hết số lượt cho phép")
```

Ba chi tiết trong đoạn này đáng để ý hơn vẻ ngoài của chúng.

Thứ nhất, điều kiện dừng mặc định là "model không gọi tool nữa". Tức là model tự tuyên bố xong việc, không có ai kiểm tra. Rất nhiều kiểu hỏng của agent bắt nguồn từ đây, và vòng kiểm chứng ở phần sau sinh ra để vá chỗ này.

Thứ hai, lỗi được trả về cho model dưới dạng kết quả tool chứ không ném exception ra ngoài. Model đọc được stack trace thì mới tự sửa hướng được.

Thứ ba, `messages` chỉ dài ra. Mỗi lượt gọi model gửi lại toàn bộ lịch sử, nên chi phí và độ nhiễu tăng theo số lượt. Hai hàm `compact` và `truncate` chỉ là hai cái tên trong pseudo-code, nhưng ở harness thật chúng là cả một mảng việc.

## Từ 30 dòng đến một harness thật

Đoạn code trên chạy được, và vẫn cách một sản phẩm dùng được rất xa. Phần chênh lệch nằm ở chín bộ phận bọc quanh vòng lặp:

```text
                         NGƯỜI DÙNG
                             |  yêu cầu, ngắt giữa chừng, duyệt quyền
+----------------------------v---------------------------------------+
|  HARNESS                                                           |
|                                                                    |
|  [1] system prompt     [8] memory file (CLAUDE.md / AGENTS.md)     |
|  [2] tool definitions        |                                     |
|          |                   |    [4] quản lý context              |
|          v                   v        (compaction, cắt kết quả)    |
|     +---------------- context window ----------------+             |
|     |  chỉ dẫn + mô tả tool + lịch sử + kết quả tool  |            |
|     +------------------------+------------------------+            |
|                              v                                     |
|                        +-----------+                               |
|      [3] vòng lặp ---> |   MODEL   | ---- văn bản ---> [9] kiểm    |
|          ^             +-----+-----+                   chứng: test,|
|          |                   | tool call               lint, review|
|          |                   v                              |      |
|          |      [5] permission + [6] hooks (trước tool)     |      |
|          |                   v                              |      |
|          |      chạy tool trong sandbox                     |      |
|          |      hoặc giao cho [7] sub-agent                 |      |
|          |                   v                              |      |
|          +----- kết quả ---- [6] hooks (sau tool)           |      |
|          +----- chưa đạt, làm tiếp -------------------------+      |
+--------------------------------------------------------------------+
```

Dưới đây mình đi qua từng bộ phận theo cùng một câu hỏi: thiếu nó thì hỏng kiểu gì.

### 1. System prompt

System prompt định vai, quy ước làm việc và giới hạn. Kiểu hỏng có hai đầu. Viết quá cứng (một chuỗi if-else bằng lời) thì giòn, gặp tình huống ngoài kịch bản là lạc. Viết quá mơ hồ thì model phải đoán. Anthropic gọi điểm cân bằng là "right altitude" và khuyên bắt đầu bằng prompt tối thiểu, rồi chỉ thêm chỉ dẫn khi quan sát thấy lỗi cụ thể ([Effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)). Nghĩa là mỗi dòng trong system prompt nên truy được về một lỗi đã thấy, không phải về một nỗi lo.

### 2. Tool definitions

Model không thấy code của tool. Nó chỉ thấy tên, mô tả và schema tham số. Với model, phần mô tả đó chính là giao diện, Anthropic gọi là agent-computer interface (ACI) và khuyên đầu tư vào nó ngang với phần prompt chính ([Building effective agents](https://www.anthropic.com/engineering/building-effective-agents)).

Hai ý thực hành rút ra từ bài đó:

- **Poka-yoke tham số**, tức đổi thiết kế để khó dùng sai. Khi làm agent cho SWE-bench, nhóm Anthropic thấy model hay dùng sai đường dẫn tương đối sau khi đổi thư mục, nên sửa tool để bắt buộc đường dẫn tuyệt đối. Họ kể rằng đã dành nhiều thời gian tối ưu tool hơn là prompt tổng.
- **Viết mô tả như docstring cho một lập trình viên mới vào nhóm**: ví dụ dùng, trường hợp biên, ranh giới với tool khác.

Về số lượng, hướng dẫn của OpenAI có một quan sát hữu ích: vấn đề nằm ở độ chồng lấn chứ không nằm ở con số. Có hệ thống chạy tốt với hơn 15 tool tách bạch rõ, có hệ thống chật vật với chưa đến 10 tool na ná nhau. Phép thử của Anthropic: nếu một kỹ sư đọc danh sách tool mà không nói chắc được tình huống này dùng cái nào, thì model cũng vậy.

### 3. Vòng lặp và điều kiện dừng

Ngoài "model không gọi tool nữa", harness thật cần thêm lối thoát: giới hạn số lượt, một output có cấu trúc đánh dấu hoàn thành, lỗi không phục hồi được, và người dùng ngắt giữa chừng. Phần ngắt giữa chừng nghe như chi tiết giao diện nhưng thật ra là thiết kế vòng lặp: harness phải hủy được tool đang chạy và chèn được chỉ dẫn mới vào giữa lượt mà không làm hỏng trạng thái hội thoại.

### 4. Quản lý context

Hội thoại dài ra thì đến lúc chạm trần context window. Trước đó khá lâu, chất lượng đã giảm vì thông tin quan trọng bị loãng. Harness xử lý bằng compaction (tóm tắt lịch sử rồi chạy tiếp với bản tóm tắt), cắt hoặc xóa kết quả tool cũ, và nạp thông tin theo nhu cầu thay vì nạp hết từ đầu. Cái giá: tài liệu Claude Code tự cảnh báo rằng sau compaction, chỉ dẫn chi tiết ở đầu hội thoại có thể mất, nên quy tắc cần bền thì phải nằm ở chỗ được nạp lại mỗi phiên.

### 5. Permission và sandbox

Tool có quyền ghi thì sai một lần là có hậu quả thật. Lớp phòng vệ thường gồm ba tầng:

- **Luật tĩnh**: danh sách cho phép và danh sách cấm theo từng tool hoặc từng mẫu câu lệnh.
- **Hỏi người** trước hành động rủi ro.
- **Sandbox**: giới hạn hệ thống file và mạng ở tầng hệ điều hành, để kể cả khi hai tầng trên sót thì thiệt hại vẫn bị chặn.

OpenAI đề xuất gán mức rủi ro thấp, vừa, cao cho từng tool theo bốn yếu tố: chỉ đọc hay có ghi, có hoàn tác được không, cần quyền tài khoản gì, có ảnh hưởng tài chính không. Mức cao thì dừng lại kiểm tra hoặc chuyển cho người.

Điểm cần nhớ là **hoàn tác được đến đâu**. Claude Code chụp lại file trước khi sửa để có thể quay lui, nhưng tài liệu của nó nói rõ những hành động chạm vào hệ thống bên ngoài (database, API, deploy) thì không quay lui được. Ranh giới giữa "sửa file cục bộ" và "tác động ra ngoài" là chỗ nên đặt chốt chặn chặt nhất.

### 6. Hooks

Hook là đoạn code của bạn mà harness chạy ở các điểm cố định trong vòng lặp: trước tool call, sau tool call, khi model định kết thúc lượt, khi phiên bắt đầu. Lý do tồn tại của nó được tài liệu Claude Code tóm bằng một phân biệt mình thấy rất đáng mang theo ([Best practices](https://code.claude.com/docs/en/best-practices)):

| | Phi tất định | Tất định |
|---|---|---|
| Ai thực thi | Model đọc rồi tự quyết | Harness chạy, model không có quyền bỏ qua |
| Hình thức | Prompt, memory file, skill | Hook, luật cấm, cấu hình quản trị |
| Độ tin cậy | "Gần như luôn luôn" | Luôn luôn |
| Hợp với | Sở thích, quy ước có ngoại lệ | Luật không có ngoại lệ |

Ví dụ: "luôn chạy formatter sau khi sửa file" viết trong memory file thì model làm theo hầu hết các lần. Đặt thành hook sau tool sửa file thì model có nhớ hay không cũng không còn quan trọng, vì người chạy lệnh không phải là model. Tài liệu đưa ra một dấu hiệu nhận biết: nếu bạn cứ phải viết hoa, thêm chữ "QUAN TRỌNG" vào một dòng chỉ dẫn mà model vẫn sót, thì dòng đó thuộc về hook.

Chiều ngược lại cũng đúng. Hook nào cũng có lúc kích hoạt nhầm và cần người bảo trì, nên tập luật cứng phải nhỏ. Lưu ý thêm: hook mà bên trong lại gọi một model khác để phán xét thì không còn tất định nữa.

### 7. Sub-agent

Sub-agent là một vòng lặp con có context window riêng. Agent chính giao một việc, sub-agent tự chạy hàng chục tool call, rồi chỉ trả về bản tóm tắt. Giá trị chính là cô lập: toàn bộ kết quả tìm kiếm, file đã đọc, các ngõ cụt đều nằm lại trong context của sub-agent, không làm bẩn context chính. Giá trị phụ là chạy song song và giới hạn quyền (một sub-agent review chỉ có tool đọc thì không thể sửa bậy, và giới hạn này do harness áp chứ không do prompt xin).

Cái giá là token và đôi khi cả thời gian: sub-agent khởi đầu trắng, phải tự thu thập lại những gì agent chính đã biết. Anthropic báo cáo hệ thống multi-agent của họ dùng khoảng 15 lần token so với chat thường, và nhận xét rằng việc code có ít phần thật sự song song hóa được hơn việc nghiên cứu ([How we built our multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system)). OpenAI khuyên tận dụng hết một agent trước, chỉ tách khi prompt đầy nhánh rẽ hoặc model liên tục chọn sai tool.

### 8. Memory file

Mỗi phiên mới bắt đầu với context trắng. Memory file (CLAUDE.md của Claude Code, AGENTS.md là định dạng mở được nhiều công cụ hỗ trợ, xem [agents.md](https://agents.md/)) là file Markdown harness tự nạp vào đầu mỗi phiên: lệnh build và test, quy ước của dự án, những điều model không thể tự suy ra từ code.

Kiểu hỏng phổ biến là file phình to. Mỗi dòng được nạp vào mọi phiên, nên mỗi dòng chiếm một phần chú ý của model ở mọi việc, kể cả việc không liên quan. Tiêu chí lọc của tài liệu Claude Code: chỉ giữ những gì model không tự tìm ra được từ code. Bài "Harness engineering" của OpenAI (kể về việc xây một sản phẩm nội bộ gần như hoàn toàn bằng Codex) mô tả cùng hướng đó: coi AGENTS.md như mục lục ngắn trỏ sang thư mục tài liệu có cấu trúc, không phải cuốn bách khoa. Kiến thức dài và chỉ thỉnh thoảng cần thì để ở chỗ nạp theo nhu cầu.

### 9. Vòng kiểm chứng

Đây là bộ phận vá điều kiện dừng "model tự tuyên bố xong". Bài về Claude Agent SDK mô tả vòng lặp agent là thu thập context, hành động, kiểm chứng, lặp lại, và xếp các kiểu phản hồi theo độ tin cậy ([Building agents with the Claude Agent SDK](https://claude.com/blog/building-agents-with-the-claude-agent-sdk)):

1. **Luật rõ ràng**: test, typecheck, lint. Đây là loại tốt nhất vì kết quả không phụ thuộc vào phán đoán. Bài đó còn nêu một hệ quả thú vị: TypeScript cho agent nhiều tầng phản hồi hơn JavaScript.
2. **Phản hồi trực quan**: chụp màn hình hoặc render giao diện để model tự nhìn.
3. **Model khác chấm**: dùng cho tiêu chí mờ. Chính bài đó cảnh báo cách này nhìn chung không vững và tốn độ trễ.

Bài của Anthropic về agent chạy dài qua nhiều context window ghi lại đúng kiểu hỏng mà thiếu kiểm chứng gây ra: model đánh dấu tính năng là xong sau khi chỉ chạy unit test hoặc gọi thử bằng curl, và ở phiên sau, một agent nhìn thấy tiến độ sẵn có rồi tuyên bố cả dự án đã hoàn tất. Cách họ sửa là một file danh sách tính năng dạng JSON, mọi mục ban đầu ở trạng thái chưa đạt, agent chỉ được sửa đúng trường trạng thái, cộng với yêu cầu kiểm thử đầu-cuối bằng trình duyệt tự động ([Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents)).

Ghép với mục hooks: một lệnh kiểm tra ghi trong memory file là lời nhắc, còn một hook chặn việc kết thúc lượt khi typecheck chưa qua mới là cưỡng chế.

## Vì sao cùng một model, harness khác nhau cho kết quả khác nhau

Nhìn lại sơ đồ sẽ thấy lý do mang tính cơ chế. Thứ model nhận ở mỗi lượt là context window, và toàn bộ nội dung trong đó do harness quyết định: chỉ dẫn nào, tool mô tả ra sao, kết quả tool bị cắt thế nào, lịch sử được tóm tắt hay giữ nguyên. Thứ model nhận lại sau mỗi hành động cũng do harness quyết định: có thông báo lỗi rõ ràng không, có bị buộc chạy test không. Cuối cùng, harness quyết định khi nào dừng. Trọng số của model giữ nguyên, nhưng đầu vào, phản hồi và điều kiện dừng đều đổi, nên kết quả đổi.

Về bằng chứng định lượng, một bài lập trường trên arXiv tháng 5/2026 gom được vài con số ([Stop Comparing LLM Agents Without Disclosing the Harness](https://arxiv.org/abs/2605.23950)):

- Trích một nghiên cứu khác: giữ nguyên model, chỉ đổi harness, pass@1 trên Terminal-Bench 2 tăng từ 69,7% lên 77,0%.
- Trích một bảng xếp hạng SWE-Bench Pro: Claude Opus 4.5 đạt 45,9% dưới scaffold chuẩn hóa và 55,4% khi chạy trong Claude Code.
- Thí nghiệm của chính nhóm tác giả (3 model, 3 cấu hình harness, 100 bài từ SWE-bench Verified): phương sai do harness lớn gấp 7,8 lần phương sai do model.

Đọc các con số này cần dè dặt. Đây là bài lập trường, tức được viết để bảo vệ một luận điểm. Hai con số đầu là trích lại từ nguồn khác. Thí nghiệm riêng chỉ có 100 bài, với các model cùng tầm năng lực, và chính tác giả ghi rõ họ không khẳng định tỉ lệ 7,8 lần là phổ quát. Kết luận mình cho là an toàn: giữa các model cùng tầm, trên việc dài hơi, chênh lệch do harness đủ lớn để một bảng xếp hạng không ghi harness là bảng xếp hạng thiếu thông tin. Nó không có nghĩa là harness tốt cứu được model yếu.

Hệ quả thực hành: khi một agent làm việc kém, đừng vội đổi model. Hãy đọc lại transcript và hỏi theo thứ tự: model có đủ thông tin không, tool có trả về thứ nó cần không, nó có cách tự kiểm tra không.

## Workflow và agent

Anthropic chia hệ thống dùng LLM thành hai loại. **Workflow** là hệ thống trong đó LLM và tool được điều phối qua các đường đi định sẵn trong code. **Agent** là hệ thống trong đó LLM tự điều khiển tiến trình và cách dùng tool của chính nó. Khác biệt cốt lõi: ai nắm luồng điều khiển, code của bạn hay model.

Năm mẫu workflow trong bài "Building effective agents":

| Mẫu | Cơ chế | Hợp khi |
|---|---|---|
| Prompt chaining | Chuỗi lời gọi nối tiếp, có thể chèn cổng kiểm tra bằng code giữa các bước | Việc tách được thành bước cố định |
| Routing | Phân loại đầu vào rồi chuyển sang nhánh chuyên biệt | Các loại đầu vào khác nhau rõ rệt, ví dụ câu dễ cho model nhỏ |
| Parallelization | Chia phần độc lập chạy song song, hoặc chạy nhiều lần rồi biểu quyết | Việc con độc lập, hoặc cần nhiều góc nhìn |
| Orchestrator-workers | Một LLM tự chia việc tùy đầu vào và giao cho các worker | Không biết trước cần chia thành những việc con nào |
| Evaluator-optimizer | Một lời gọi sinh, một lời gọi chấm và góp ý, lặp lại | Có tiêu chí đánh giá rõ, lặp có cải thiện đo được |

Cần cẩn thận với thuật ngữ khi đọc chéo nguồn. Trong hướng dẫn của OpenAI, "workflow" chỉ chuỗi bước cần làm để đạt mục tiêu của người dùng, và agent là hệ thống dùng LLM để điều khiển việc thực thi workflow đó. Ứng dụng có LLM nhưng không để LLM điều khiển luồng (chatbot đơn giản, bộ phân loại cảm xúc) thì theo họ không phải agent. Hai bên dùng cùng một từ cho hai thứ khác nhau, nhưng tiêu chí nhận diện agent thì trùng: model nắm luồng điều khiển.

Trong thực tế hai loại này lồng nhau. Một coding agent chạy trong CI là agent nằm trong một workflow. Một agent gọi sub-agent theo mẫu orchestrator-workers là workflow nằm trong agent.

## Khi nào không nên dùng agent

Agent đổi độ trễ và chi phí lấy tính linh hoạt, và lỗi của nó cộng dồn qua từng bước. Anthropic khuyên tìm giải pháp đơn giản nhất và chỉ thêm độ phức tạp khi nó cải thiện kết quả một cách chứng minh được. Với nhiều ứng dụng, tối ưu một lời gọi LLM đơn kèm retrieval và ví dụ trong prompt là đủ.

Những dấu hiệu nên dừng ở workflow hoặc một lời gọi đơn:

- **Các bước đã biết trước.** Nếu bạn vẽ được flowchart của công việc thì hãy code flowchart đó. Workflow dễ test, dễ đoán chi phí, dễ debug hơn.
- **Không có cách kiểm chứng tự động.** Agent mạnh ở coding một phần vì có test để tự sửa. Việc mà chỉ người mới đánh giá được đúng sai thì mỗi vòng lặp của agent đều chạy mù.
- **Sai một lần là đắt và không hoàn tác được.** Chuyển tiền, gửi email cho khách hàng, xóa dữ liệu. Nếu vẫn muốn dùng, hành động đó phải nằm sau chốt duyệt của người.
- **Yêu cầu độ trễ thấp hoặc chi phí mỗi lượt phải đoán trước được.** Số lượt của agent không cố định.
- **Luật nghiệp vụ đơn giản và ổn định.** OpenAI nêu ba nhóm việc đáng dùng agent: quyết định cần phán đoán theo ngữ cảnh, bộ luật đã rối đến mức khó bảo trì, và phụ thuộc nặng vào dữ liệu phi cấu trúc. Họ nói rõ nếu không rơi vào các nhóm này thì giải pháp tất định có thể là đủ.

Chiều ngược lại, agent đáng dùng khi bài toán mở, không đoán trước được số bước, và môi trường cho phản hồi đáng tin ở mỗi bước.

## Checklist khi tự dựng hoặc cấu hình harness

- Điều kiện dừng: ngoài việc model tự dừng, đã có giới hạn số lượt và một phép kiểm tra khách quan chưa?
- Lỗi của tool có quay về cho model dưới dạng đọc hiểu được, kèm gợi ý sửa không?
- Mỗi tool: một kỹ sư mới đọc mô tả có biết khi nào dùng và khi nào không? Có cặp tool nào chồng lấn?
- Kết quả tool có bị giới hạn kích thước trước khi vào context không?
- Hành động nào không hoàn tác được? Chúng đang sau luật cứng, sau chốt duyệt của người, hay chỉ sau một dòng dặn dò trong prompt?
- Luật không có ngoại lệ đã thành hook hoặc luật cấm chưa? Số luật cứng có đủ ít để bảo trì không?
- Memory file: mỗi dòng có phải thứ model không tự tìm ra được từ code?
- Sub-agent: lời giao việc có đủ mục tiêu, định dạng kết quả, ranh giới nhiệm vụ không?
- Có lưu transcript đầy đủ để đọc lại khi hỏng không? Không có transcript thì không debug được harness.

## Điều còn mở, chưa chắc

- **Harness nên dày hay mỏng khi model mạnh lên?** Một hướng cho rằng model càng giỏi thì harness càng nên mỏng, nhiều giàn giáo hôm nay sẽ thành thừa. Hướng kia cho rằng việc càng dài hơi thì càng cần cấu trúc quanh model. Chính bài về agent chạy dài của Anthropic để ngỏ câu hỏi một agent đa năng hay nhiều agent chuyên biệt (test, QA, dọn dẹp) thì tốt hơn.
- **Đo đóng góp của harness thế nào cho công bằng.** Bằng chứng định lượng hiện còn mỏng, phần lớn đến từ bảng xếp hạng và bài của các bên có lợi ích liên quan. Chưa có chuẩn công bố cấu hình harness kèm kết quả benchmark.
- **Multi-agent cho coding.** Anthropic báo cáo lợi ích rõ ở việc nghiên cứu, đồng thời nói coding ít phần song song hóa được. OpenAI khuyên vắt kiệt một agent trước. Ranh giới nằm ở đâu thì chưa ai có câu trả lời chung.
- **Dùng model làm lớp permission.** Một bộ phân loại duyệt hành động giúp bớt hỏi người, nhưng nó là phán đoán chứ không phải luật, tức có thể sai. Cân bằng giữa tiện và chắc vẫn đang được dò.
- **Giới hạn của bài này.** Phần lớn nguồn là từ chính các hãng làm model và làm harness, viết về sản phẩm của họ. Nhiều phần của các harness thương mại không công khai, nên bản đồ ở trên dựng từ những gì họ chọn kể ra.

## Nguồn đọc thêm

- [Building effective agents](https://www.anthropic.com/engineering/building-effective-agents) (Anthropic, 12/2024): định nghĩa workflow và agent, năm mẫu workflow, phụ lục về thiết kế tool. Nên đọc đầu tiên.
- [A practical guide to building agents](https://cdn.openai.com/business-guides-and-resources/a-practical-guide-to-building-agents.pdf) (OpenAI, bản PDF): tiêu chí khi nào nên dựng agent, phân loại guardrail, mẫu manager và handoff. Đọc để thấy cách dùng từ khác với Anthropic.
- [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works) và [Best practices](https://code.claude.com/docs/en/best-practices): tài liệu của một harness đang chạy thật. Phần phân biệt chỉ dẫn tất định và phi tất định đáng đọc kể cả khi bạn dùng công cụ khác.
- [Building agents with the Claude Agent SDK](https://claude.com/blog/building-agents-with-the-claude-agent-sdk) (Anthropic, 9/2025): vòng thu thập context, hành động, kiểm chứng, và thứ bậc các kiểu phản hồi.
- [Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents) (Anthropic, 11/2025): các kiểu hỏng khi agent làm việc qua nhiều context window và cách họ vá bằng file trạng thái.
- [How we built our multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system) (Anthropic, 6/2025): chi phí token và điều kiện để multi-agent có lợi.
- [Stop Comparing LLM Agents Without Disclosing the Harness](https://arxiv.org/abs/2605.23950) (arXiv, 5/2026): lập luận và số liệu về ảnh hưởng của harness lên benchmark. Đọc với tâm thế phản biện.
- [agents.md](https://agents.md/): mô tả định dạng AGENTS.md và danh sách công cụ hỗ trợ.
- "Harness engineering: leveraging Codex in an agent-first world" (OpenAI, 2/2026): bài gốc của cụm từ harness engineering theo nghĩa đang được dùng rộng. Không kèm link vì trang gốc chặn truy cập tự động khi bài này được soạn, bạn tìm theo tên trên trang của OpenAI.
