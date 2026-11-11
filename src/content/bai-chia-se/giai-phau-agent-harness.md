---
title: "Giải phẫu một agent harness: vòng lặp, chín bộ phận và vì sao cùng model mà kết quả khác nhau"
date: 2026-11-08
tags: [Harness, Agent]
description: Bản đồ tổng thể về agent harness cho kỹ sư phần mềm, gồm vòng lặp tối thiểu khoảng 30 dòng, chín bộ phận bọc quanh model, ranh giới workflow và agent, và những lúc không nên dùng agent.
loai: chuyen-sau
---

Một coding agent như Claude Code, Codex hay Cursor gồm hai phần: model sinh ra token, và mọi thứ còn lại. "Mọi thứ còn lại" là harness. Tài liệu của Claude Code định nghĩa: đó là lớp bọc quanh model, cung cấp tool và quản lý phần context mà model nhìn thấy ([How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works)).

Bài này là bản đồ: vòng lặp lõi trông thế nào, harness thật có những bộ phận gì, mỗi bộ phận chặn kiểu hỏng nào. Bốn bộ phận lớn có bài riêng, ở đây mình chỉ nêu vai trò rồi dẫn link: [quản lý context](/bai-chia-se/context-engineering-cho-agent/), [thiết kế tool, MCP và Skills](/bai-chia-se/thiet-ke-tool-mcp-skill-cho-agent/), [multi-agent và agent chạy dài](/bai-chia-se/multi-agent-va-agent-chay-dai/), [bảo mật](/bai-chia-se/bao-mat-agent-prompt-injection/).

Quy ước trong bài: câu nào có tên nguồn là nguồn nói. Câu bắt đầu bằng "mình suy ra", "mình cho rằng" là suy luận của người viết, bạn nên kiểm lại trên hệ thống của mình.

## Vòng lặp tối thiểu

Lõi của mọi agent là một vòng `while`: gọi model, nếu model yêu cầu tool call (model trả về tên hàm và tham số thay vì văn bản) thì chạy tool, nối kết quả vào hội thoại, gọi lại. Hướng dẫn của OpenAI viết rằng khái niệm vòng lặp while này là trung tâm của cách agent vận hành, và agent chạy đến khi gặp điều kiện thoát ([A practical guide to building agents](https://cdn.openai.com/business-guides-and-resources/a-practical-guide-to-building-agents.pdf)).

Pseudo-code kiểu Python dưới đây do mình viết để minh họa, không gắn với SDK nào:

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

Ba chi tiết đáng để ý:

1. Điều kiện dừng mặc định là "model không gọi tool nữa". Model tự tuyên bố xong việc, không ai kiểm tra. Vòng kiểm chứng ở mục 9 sinh ra để vá chỗ này.
2. Lỗi được trả về cho model dưới dạng kết quả tool chứ không ném exception ra ngoài. Model đọc được thông báo lỗi thì mới tự sửa hướng được.
3. `messages` chỉ dài ra. Mỗi lượt gửi lại toàn bộ lịch sử, nên chi phí và độ nhiễu tăng theo số lượt. `compact` và `truncate` ở đây là hai cái tên, ở harness thật chúng là cả một mảng việc.

## Từ 30 dòng đến một harness thật

Đoạn code trên chạy được nhưng chưa dùng được. Phần chênh lệch nằm ở chín bộ phận bọc quanh vòng lặp. Cách chia thành chín là của mình, gom từ tài liệu Claude Code và hai bản hướng dẫn của Anthropic và OpenAI; không nguồn nào liệt kê đúng danh sách này.

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

Mỗi bộ phận dưới đây trả lời cùng một câu hỏi: thiếu nó thì hỏng kiểu gì.

### 1. System prompt

System prompt định vai, quy ước làm việc và giới hạn. Kiểu hỏng có hai đầu: viết quá cứng (một chuỗi if-else bằng lời) thì giòn, viết quá mơ hồ thì model phải đoán. Anthropic gọi điểm cân bằng là "right altitude" và khuyên bắt đầu bằng prompt tối thiểu, rồi thêm chỉ dẫn dựa trên các kiểu lỗi quan sát được khi thử ([Effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)). Mình suy ra một phép thử: mỗi dòng trong system prompt nên truy được về một lỗi đã thấy.

### 2. Tool definitions

Model không thấy code của tool, chỉ thấy tên, mô tả và schema tham số. Anthropic gọi lớp này là agent-computer interface (ACI) và khuyên đầu tư vào nó ngang với giao diện cho người ([Building effective agents](https://www.anthropic.com/engineering/building-effective-agents)). Thiếu đầu tư ở đây thì model chọn nhầm tool hoặc điền sai tham số, và lỗi đó thường bị đổ cho model.

Về số lượng, hướng dẫn của OpenAI ghi nhận có hệ thống chạy tốt với hơn 15 tool tách bạch rõ, có hệ thống chật vật với chưa đến 10 tool chồng lấn nhau. Phép thử của Anthropic: nếu một kỹ sư đọc danh sách tool mà không nói chắc được tình huống này dùng cái nào, thì đừng mong model làm tốt hơn. Cách viết mô tả, tham số, kết quả trả về và thông báo lỗi nằm ở bài [Thiết kế tool cho agent](/bai-chia-se/thiet-ke-tool-mcp-skill-cho-agent/).

### 3. Vòng lặp và điều kiện dừng

Ngoài "model không gọi tool nữa", harness cần thêm lối thoát: giới hạn số lượt, một output có cấu trúc đánh dấu hoàn thành, lỗi không phục hồi được, và người dùng ngắt giữa chừng. Tài liệu Claude Code mô tả người dùng có thể dừng ngay (tool đang chạy bị hủy) hoặc gõ thêm chỉ dẫn để model đọc khi các tool call hiện tại xong. Mình cho rằng đây là thiết kế vòng lặp chứ không phải chi tiết giao diện: harness phải hủy được tool và chèn được tin nhắn vào giữa lượt mà không làm hỏng trạng thái hội thoại.

### 4. Quản lý context

Hội thoại dài ra thì chạm trần context window, và chất lượng thường giảm từ trước đó vì thông tin quan trọng bị loãng. Harness xử lý bằng compaction (tóm tắt lịch sử rồi chạy tiếp), xóa kết quả tool cũ, và nạp thông tin theo nhu cầu. Cái giá, theo chính tài liệu Claude Code: sau compaction, chỉ dẫn chi tiết ở đầu hội thoại có thể mất, nên quy tắc cần bền phải nằm ở memory file. Cơ chế và trade-off của từng kỹ thuật ở bài [Context engineering cho agent](/bai-chia-se/context-engineering-cho-agent/).

### 5. Permission và sandbox

Tool có quyền ghi thì sai một lần là có hậu quả thật. Mình gom lớp phòng vệ thành ba tầng:

- **Luật tĩnh**: danh sách cho phép và danh sách cấm theo tool hoặc mẫu câu lệnh.
- **Hỏi người** trước hành động rủi ro.
- **Sandbox**: giới hạn hệ thống file và mạng ở tầng hệ điều hành, để khi hai tầng trên sót thì thiệt hại vẫn bị chặn.

OpenAI đề xuất gán mức rủi ro thấp, vừa, cao cho từng tool theo các yếu tố như chỉ đọc hay có ghi, có hoàn tác được không, cần quyền tài khoản gì, có ảnh hưởng tài chính không.

Điểm cần nhớ là hoàn tác được đến đâu. Claude Code chụp lại file trước khi sửa để quay lui, nhưng tài liệu nói rõ hành động chạm vào hệ thống bên ngoài (database, API, deploy) thì không quay lui được. Mình suy ra: ranh giới giữa "sửa file cục bộ" và "tác động ra ngoài" là chỗ đặt chốt chặn chặt nhất. Phần tấn công qua nội dung bẩn và cách phòng thủ theo lớp ở bài [Bảo mật cho agent](/bai-chia-se/bao-mat-agent-prompt-injection/).

### 6. Hooks

Hook là đoạn code của bạn mà harness chạy ở các điểm cố định trong vòng lặp: trước tool call, sau tool call, khi model định kết thúc lượt, khi phiên bắt đầu. Tài liệu Claude Code giải thích lý do tồn tại của nó bằng phân biệt giữa chỉ dẫn phi tất định và tất định ([Best practices](https://code.claude.com/docs/en/best-practices)). Bảng dưới là bản mình tóm lại:

| | Phi tất định | Tất định |
|---|---|---|
| Ai thực thi | Model đọc rồi tự quyết | Harness chạy, model không bỏ qua được |
| Hình thức | Prompt, memory file, skill | Hook, luật cấm, cấu hình quản trị |
| Độ tin cậy | Thường làm theo, không bảo đảm | Mọi lần |
| Hợp với | Sở thích, quy ước có ngoại lệ | Luật không có ngoại lệ |

Ví dụ của tài liệu: "luôn chạy formatter sau khi sửa file" viết trong memory file thì model làm theo hầu hết các lần. Đặt thành hook sau tool sửa file thì model có nhớ hay không cũng không còn quan trọng, vì bên chạy lệnh không phải là model. Dấu hiệu nhận biết họ đưa ra: nếu bạn cứ phải thêm nhấn mạnh vào một dòng chỉ dẫn mà model vẫn sót, dòng đó thuộc về hook.

Ba giới hạn, cũng từ tài liệu đó. Hook nào cũng có lúc kích hoạt nhầm và cần người bảo trì, nên tập luật cứng phải nhỏ. Hook chỉ là sở thích thì nên xóa và để model tự cân nhắc. Hook gọi một model khác để phán xét thì không còn tất định.

### 7. Sub-agent

Sub-agent là một vòng lặp con có context window riêng: agent chính giao một việc, sub-agent tự chạy nhiều tool call rồi chỉ trả về bản tóm tắt. Thiếu nó, mọi kết quả tìm kiếm, file đã đọc và ngõ cụt đều nằm lại trong context chính. Harness còn giới hạn được quyền của sub-agent ở tầng cấu hình, ví dụ một sub-agent review chỉ có tool đọc.

Cái giá là token và rủi ro phối hợp. Khi nào nên tách, tách kiểu gì, và viết lời giao việc thế nào ở bài [Multi-agent và agent chạy dài](/bai-chia-se/multi-agent-va-agent-chay-dai/).

### 8. Memory file

Mỗi phiên mới bắt đầu với context trắng. Memory file (CLAUDE.md của Claude Code; AGENTS.md là định dạng mở được nhiều công cụ hỗ trợ, xem [agents.md](https://agents.md/)) là file Markdown harness tự nạp vào đầu mỗi phiên: lệnh build và test, quy ước của dự án.

Kiểu hỏng phổ biến là file phình to. Mỗi dòng được nạp vào mọi phiên, kể cả phiên làm việc không liên quan. Tiêu chí lọc của tài liệu Claude Code: giữ những gì model không tự lấy được từ việc đọc repo. Tài liệu cũng nói thêm chữ "IMPORTANT" vào một dòng hay bị sót thường không giúp gì; hoặc file đã quá dài, hoặc dòng đó là luật cứng và thuộc về hook.

Bài "Harness engineering" của OpenAI (2/2026, kể về việc xây một sản phẩm nội bộ bằng Codex) được dẫn lại nhiều với cùng hướng: giữ AGENTS.md ngắn như một mục lục trỏ sang thư mục tài liệu có cấu trúc. Lưu ý về độ tin cậy: trang gốc chặn truy cập tự động nên mình chưa đọc được nguyên văn, chỉ đối chiếu được ngày đăng và ý này qua kết quả tìm kiếm. Bạn nên đọc bài gốc trước khi trích lại.

### 9. Vòng kiểm chứng

Đây là bộ phận vá điều kiện dừng "model tự tuyên bố xong". Bài về Claude Agent SDK mô tả vòng lặp agent là thu thập context, hành động, kiểm chứng, lặp lại, và nêu ba kiểu phản hồi ([Building agents with the Claude Agent SDK](https://claude.com/blog/building-agents-with-the-claude-agent-sdk)):

1. **Luật rõ ràng**: test, typecheck, lint. Bài đó gọi đây là dạng phản hồi tốt nhất, và nêu một hệ quả: sinh TypeScript rồi lint cho agent nhiều tầng phản hồi hơn sinh JavaScript thuần.
2. **Phản hồi trực quan**: chụp màn hình hoặc render giao diện để model tự nhìn.
3. **Model khác chấm**: dùng cho tiêu chí mờ. Chính bài đó cảnh báo cách này nhìn chung không vững và tốn độ trễ.

Thiếu kiểm chứng thì hỏng thế nào đã được Anthropic ghi lại ở agent chạy qua nhiều phiên: model đánh dấu tính năng là xong khi chưa thử đầu-cuối, và phiên sau thấy đã có tiến độ thì tuyên bố cả dự án hoàn tất ([Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents)). Cách họ sửa nằm ở bài [Multi-agent và agent chạy dài](/bai-chia-se/multi-agent-va-agent-chay-dai/).

Ghép với mục hooks: một lệnh kiểm tra ghi trong memory file là lời nhắc; một hook chặn việc kết thúc lượt khi typecheck chưa qua mới là cưỡng chế. Vòng kiểm chứng trong lúc chạy khác với việc đo chất lượng agent trên một bộ task cố định, phần sau ở bài [Eval cho ứng dụng LLM và agent](/bai-chia-se/eval-cho-agent/).

## Vì sao cùng một model, harness khác nhau cho kết quả khác nhau

Lập luận cơ chế dưới đây là của mình, dựng từ sơ đồ. Thứ model nhận ở mỗi lượt là context window, và nội dung trong đó do harness quyết định: chỉ dẫn nào, tool mô tả ra sao, kết quả tool bị cắt thế nào, lịch sử được tóm tắt hay giữ nguyên. Thứ model nhận lại sau mỗi hành động cũng do harness quyết định: có thông báo lỗi rõ không, có bị buộc chạy test không. Harness còn quyết định khi nào dừng. Trọng số giữ nguyên, nhưng đầu vào, phản hồi và điều kiện dừng đều đổi.

Về bằng chứng định lượng, một bài lập trường trên arXiv tháng 5/2026 gom được vài con số ([Stop Comparing LLM Agents Without Disclosing the Harness](https://arxiv.org/abs/2605.23950)):

- Trích một nghiên cứu khác (AHE, arXiv 2604.25850): giữ nguyên model GPT-5.4, chỉ đổi harness, pass@1 trên Terminal-Bench 2 tăng từ 69,7% lên 77,0%.
- Trích một bảng xếp hạng SWE-Bench Pro của bên thứ ba: Claude Opus 4.5 đạt 45,9% dưới scaffold chuẩn hóa và 55,4% khi chạy trong Claude Code.
- Thí nghiệm của chính nhóm tác giả: 3 model (GPT-5.4, Kimi K2.6, GLM-5.1), 3 cấu hình harness, 100 bài chọn từ SWE-bench Verified, mỗi ô chạy 2 lần. Phương sai do harness lớn gấp 7,8 lần phương sai do model, và thứ hạng giữa các model đảo chiều ở 6 trên 9 phép so sánh.

Đọc các con số này cần dè dặt. Bài lập trường được viết để bảo vệ một luận điểm. Hai con số đầu là trích lại, mình chưa đối chiếu nguồn cấp một. Thí nghiệm riêng nhỏ, và ba model được chọn vì sát nhau trên bảng xếp hạng, tức thiết kế đã thu hẹp sẵn phương sai do model. Tác giả ghi rõ họ không khẳng định tỉ lệ 7,8 lần là phổ quát. Kết luận mình cho là an toàn: giữa các model cùng tầm, trên việc dài hơi, chênh lệch do harness đủ lớn để một bảng xếp hạng không ghi harness là bảng xếp hạng thiếu thông tin. Các con số này không nói rằng harness tốt cứu được model yếu.

Hệ quả thực hành, theo mình: khi một agent làm việc kém, đọc transcript trước khi đổi model, và hỏi theo thứ tự: model có đủ thông tin không, tool có trả về thứ nó cần không, nó có cách tự kiểm tra không.

## Workflow và agent

Anthropic chia hệ thống dùng LLM thành hai loại. **Workflow** là hệ thống trong đó LLM và tool được điều phối qua các đường đi định sẵn trong code. **Agent** là hệ thống trong đó LLM tự điều khiển tiến trình và cách dùng tool của chính nó. Khác biệt cốt lõi: ai nắm luồng điều khiển, code của bạn hay model.

Năm mẫu workflow trong bài "Building effective agents":

| Mẫu | Cơ chế | Hợp khi |
|---|---|---|
| Prompt chaining | Chuỗi lời gọi nối tiếp, có thể chèn cổng kiểm tra bằng code giữa các bước | Việc tách được thành bước cố định |
| Routing | Phân loại đầu vào rồi chuyển sang nhánh chuyên biệt | Các loại đầu vào khác nhau rõ rệt |
| Parallelization | Chia phần độc lập chạy song song, hoặc chạy nhiều lần rồi biểu quyết | Việc con độc lập, hoặc cần nhiều góc nhìn |
| Orchestrator-workers | Một LLM tự chia việc tùy đầu vào và giao cho các worker | Không biết trước cần chia thành những việc con nào |
| Evaluator-optimizer | Một lời gọi sinh, một lời gọi chấm và góp ý, lặp lại | Có tiêu chí đánh giá rõ, lặp có cải thiện đo được |

Cẩn thận với thuật ngữ khi đọc chéo nguồn. Trong hướng dẫn của OpenAI, "workflow" là chuỗi bước cần thực hiện để đạt mục tiêu của người dùng, và agent là hệ thống dùng LLM để điều khiển việc thực thi workflow đó. Ứng dụng có LLM nhưng không để LLM điều khiển luồng (chatbot đơn giản, bộ phân loại cảm xúc) thì theo họ không phải agent. Hai bên dùng cùng một từ cho hai thứ khác nhau, nhưng tiêu chí nhận diện agent thì trùng: model nắm luồng điều khiển.

Theo quan sát của mình, hai loại này lồng nhau trong thực tế. Một coding agent chạy trong CI là agent nằm trong một workflow. Một agent gọi sub-agent theo mẫu orchestrator-workers là workflow nằm trong agent.

## Khi nào không nên dùng agent

Anthropic khuyên tìm giải pháp đơn giản nhất và chỉ thêm độ phức tạp khi cần; với nhiều ứng dụng, tối ưu một lời gọi LLM đơn kèm retrieval và ví dụ trong prompt là đủ. OpenAI nêu ba nhóm việc đáng dùng agent: quyết định cần phán đoán theo ngữ cảnh, bộ luật đã rối đến mức khó bảo trì, và phụ thuộc nặng vào dữ liệu phi cấu trúc. Họ viết rằng ngoài các nhóm này, giải pháp tất định có thể là đủ.

Từ hai nguồn đó và từ cấu trúc vòng lặp ở trên, mình rút ra các dấu hiệu nên dừng ở workflow hoặc một lời gọi đơn:

- **Các bước đã biết trước.** Vẽ được flowchart của công việc thì code flowchart đó. Workflow dễ test, dễ đoán chi phí, dễ debug hơn.
- **Không có cách kiểm chứng tự động.** Agent mạnh ở coding một phần vì có test để tự sửa. Việc chỉ người mới đánh giá được đúng sai thì mỗi vòng lặp chạy mù.
- **Sai một lần là đắt và không hoàn tác được.** Chuyển tiền, gửi email cho khách hàng, xóa dữ liệu. Nếu vẫn dùng agent, hành động đó phải nằm sau chốt duyệt của người.
- **Cần độ trễ thấp hoặc chi phí mỗi lượt đoán trước được.** Số lượt của agent không cố định.

Chiều ngược lại, agent đáng dùng khi bài toán mở, không đoán trước được số bước, và môi trường cho phản hồi đáng tin ở mỗi bước.

## Checklist khi tự dựng hoặc cấu hình harness

- Điều kiện dừng: ngoài việc model tự dừng, đã có giới hạn số lượt và một phép kiểm tra khách quan chưa?
- Lỗi của tool có quay về cho model ở dạng đọc hiểu được, kèm gợi ý sửa không?
- Một kỹ sư mới đọc mô tả tool có biết khi nào dùng và khi nào không? Có cặp tool nào chồng lấn?
- Kết quả tool có bị giới hạn kích thước trước khi vào context không?
- Hành động nào không hoàn tác được? Chúng đang sau luật cứng, sau chốt duyệt của người, hay chỉ sau một dòng dặn dò trong prompt?
- Luật không có ngoại lệ đã thành hook hoặc luật cấm chưa? Số luật cứng có đủ ít để bảo trì không?
- Memory file: mỗi dòng có phải thứ model không tự lấy được từ repo?
- Có lưu transcript đầy đủ để đọc lại khi hỏng không? Không có transcript thì không debug được harness.

## Điều còn mở, chưa chắc

- **Harness nên dày hay mỏng khi model mạnh lên?** Một hướng cho rằng model càng giỏi thì nhiều giàn giáo hôm nay sẽ thành thừa. Hướng kia cho rằng việc càng dài hơi càng cần cấu trúc quanh model. Bài về agent chạy dài của Anthropic tự để ngỏ câu hỏi một agent đa năng hay nhiều agent chuyên biệt (test, QA, dọn dẹp) thì tốt hơn.
- **Đo đóng góp của harness thế nào cho công bằng.** Bằng chứng định lượng còn mỏng, phần lớn đến từ bảng xếp hạng và bài của các bên có lợi ích liên quan. Chưa có chuẩn công bố cấu hình harness kèm kết quả benchmark.
- **Dùng model làm lớp permission.** Tài liệu Claude Code mô tả chế độ dùng một bộ phân loại duyệt hành động rủi ro để bớt hỏi người, và tự ghi rằng đó là phán đoán, có thể sai, nên luật "không bao giờ" vẫn phải nằm ở luật cấm hoặc hook. Cân bằng giữa tiện và chắc vẫn đang được dò.
- **Giới hạn của bài này.** Phần lớn nguồn là từ chính các hãng làm model và làm harness, viết về sản phẩm của họ. Nhiều phần của các harness thương mại không công khai, nên bản đồ ở trên dựng từ những gì họ chọn kể ra. Chi tiết sản phẩm mô tả theo tài liệu lúc soạn bài (11/2026) và sẽ đổi.

## Nguồn đọc thêm

- [Building effective agents](https://www.anthropic.com/engineering/building-effective-agents) (Anthropic, 12/2024): định nghĩa workflow và agent, năm mẫu workflow, phụ lục về thiết kế tool. Nên đọc đầu tiên.
- [A practical guide to building agents](https://cdn.openai.com/business-guides-and-resources/a-practical-guide-to-building-agents.pdf) (OpenAI, bản PDF): tiêu chí khi nào nên dựng agent, phân mức rủi ro của tool, khi nào tách nhiều agent. Đọc để thấy cách dùng từ khác với Anthropic.
- [How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works) và [Best practices](https://code.claude.com/docs/en/best-practices): tài liệu của một harness đang chạy thật. Phần phân biệt chỉ dẫn tất định và phi tất định đáng đọc kể cả khi bạn dùng công cụ khác.
- [Building agents with the Claude Agent SDK](https://claude.com/blog/building-agents-with-the-claude-agent-sdk) (Anthropic, 9/2025): vòng thu thập context, hành động, kiểm chứng, và ba kiểu phản hồi.
- [Effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) (Anthropic, 9/2025): nguồn của "right altitude" và phép thử chồng lấn tool.
- [Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents) (Anthropic, 11/2025): các kiểu hỏng khi agent làm việc qua nhiều context window.
- [Stop Comparing LLM Agents Without Disclosing the Harness](https://arxiv.org/abs/2605.23950) (arXiv, 5/2026): lập luận và số liệu về ảnh hưởng của harness lên benchmark. Đọc với tâm thế phản biện.
- [agents.md](https://agents.md/): mô tả định dạng AGENTS.md và danh sách công cụ hỗ trợ.
- "Harness engineering: leveraging Codex in an agent-first world" (OpenAI, 2/2026): bài hay được dẫn cho cụm từ harness engineering. Không kèm link vì trang gốc chặn truy cập tự động khi bài này được soạn và soát; bạn tìm theo tên trên trang của OpenAI.
