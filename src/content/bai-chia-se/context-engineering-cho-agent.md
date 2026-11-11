---
title: "Context engineering cho agent: context rot, prompt caching và sáu kỹ thuật giữ context gọn"
date: 2026-11-09
tags: [Context engineering, Agent]
description: Vì sao context window càng đầy thì model càng kém, cách bố trí prompt để giữ cache, và các kỹ thuật compaction, ghi chú ngoài, sub-agent. Có so sánh chỗ Anthropic, Manus, Chroma và LangChain nói khác nhau, kèm checklist thiết kế.
loai: chuyen-sau
---

Một agent chạy 50 lượt tool call thì gửi cho model 50 prompt, và gần như toàn bộ nội dung của các prompt đó không do bạn viết. Nó là kết quả tool, file đã đọc, lỗi, bản tóm tắt. Context engineering là việc quyết định cái gì được nằm trong context window (phần văn bản model nhìn thấy ở một lượt gọi) tại mỗi lượt. Anthropic định nghĩa nó là tập chiến lược để tuyển chọn và duy trì bộ token tối ưu trong lúc suy luận, với nguyên tắc: tìm tập token nhỏ nhất, tín hiệu cao nhất, để tối đa hóa khả năng ra kết quả mong muốn ([Effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)).

Bài này đi vào cơ chế: vì sao "nhỏ nhất" vẫn quan trọng khi context window đã lên hàng trăm nghìn token, rồi đến sáu nhóm kỹ thuật và những chỗ các nguồn mâu thuẫn nhau. Vị trí của phần này trong toàn bộ harness xem ở bài [Giải phẫu một agent harness](/bai-chia-se/giai-phau-agent-harness/).

Quy ước: câu có tên nguồn là nguồn nói. Chỗ nào mình viết "mình suy ra", "theo mình" là suy luận của người viết.

## Vì sao context là tài nguyên hữu hạn

Có hai giới hạn. Giới hạn cứng là kích thước context window. Giới hạn mềm đến sớm hơn: chất lượng giảm dần khi đầu vào dài ra, kể cả khi còn xa mới chạm trần.

### Bằng chứng: nghiên cứu context rot của Chroma

Báo cáo [Context Rot](https://www.trychroma.com/research/context-rot) (Chroma, 7/2025) thử 18 model của Anthropic, OpenAI, Google và Alibaba (không phải model nào cũng có mặt ở mọi thí nghiệm), với thiết kế giữ nguyên độ khó của bài và chỉ thay đổi độ dài đầu vào. Kết luận chung: độ tin cậy giảm dần khi đầu vào dài ra. Các phát hiện đáng nhớ:

- **Độ giống giữa câu hỏi và đáp án càng thấp thì suy giảm càng nhanh.** Khi câu hỏi và đoạn chứa đáp án gần trùng từ ngữ, model tìm tốt hơn ở mọi độ dài. Khi phải suy diễn ngữ nghĩa, hiệu năng rơi nhanh theo độ dài.
- **Chỉ một đoạn gây nhiễu đã làm giảm kết quả.** Đoạn gây nhiễu (distractor) là đoạn cùng chủ đề, trông giống đáp án nhưng sai. Bốn đoạn thì tệ hơn nữa.
- **Bài hội thoại dài.** Trên 306 câu hỏi của LongMemEval, họ so hai cách đưa đầu vào: toàn bộ lịch sử (trung bình khoảng 113 nghìn token) và chỉ phần liên quan (khoảng 300 token). Mọi model đều làm tốt hơn ở bản rút gọn. Bật chế độ suy luận có cải thiện nhưng khoảng cách vẫn còn.
- **Một kết quả ngược trực giác.** Văn bản nền bị xáo trộn thứ tự câu cho kết quả tốt hơn văn bản mạch lạc, ở cả 18 model. Nhóm tác giả không giải thích được.

Giới hạn do chính tác giả nêu: họ không giải thích cơ chế, khó tách độ dài đầu vào khỏi độ khó của bài, và các bài thử đều đơn giản hơn công việc thật (họ cho rằng việc thật có thể suy giảm nặng hơn). Cũng nên nhớ Chroma bán cơ sở dữ liệu vector, tức có lợi ích khi kết luận nghiêng về retrieval.

Mình suy ra cho agent: kết quả tìm kiếm cũ và các phiên bản file đã lỗi thời đóng đúng vai distractor. Chroma không thử trường hợp này.

### Cách hình dung: attention budget

Anthropic đưa ra một cách nghĩ: model có một "attention budget" hữu hạn và mỗi token mới tiêu một phần của nó. Lý do kiến trúc họ nêu: transformer cho mọi token liên hệ với mọi token khác, nên n token tạo ra n² quan hệ từng đôi.

Đây là mô hình tư duy, không phải định luật đo được. Theo mình, giá trị của nó là đổi câu hỏi thiết kế từ "còn nhét vừa không" sang "token này có đáng chỗ nó chiếm không".

### Bốn kiểu hỏng của context

Bài của LangChain dẫn lại cách phân loại của Drew Breunig ([Context Engineering](https://www.langchain.com/blog/context-engineering-for-agents)). Hai cột đầu là của nguồn, cột ví dụ là mình thêm để dễ nhận ra khi đọc transcript:

| Kiểu hỏng | Chuyện gì xảy ra | Ví dụ trong agent |
|---|---|---|
| Poisoning | Một thông tin bịa lọt vào context | Model ghi nhầm tên hàm vào ghi chú, các lượt sau tin theo |
| Distraction | Context dài lấn át những gì model đã học | Model lặp lại hành động trong lịch sử thay vì nghĩ bước mới |
| Confusion | Nội dung thừa ảnh hưởng đến câu trả lời | Mô tả của 40 tool không liên quan khiến model gọi nhầm |
| Clash | Các phần trong context mâu thuẫn nhau | File đọc ở lượt 5 khác với file sau khi sửa ở lượt 20 |

## Bản đồ kỹ thuật: write, select, compress, isolate

LangChain gom các kỹ thuật vào bốn động từ:

| Nhóm | Ý nghĩa | Kỹ thuật trong bài này |
|---|---|---|
| Write | Ghi ra ngoài context window | Ghi chú có cấu trúc, file làm bộ nhớ |
| Select | Kéo đúng thứ vào | Just-in-time retrieval, memory file |
| Compress | Chỉ giữ token cần | Compaction, tool result clearing |
| Isolate | Tách context | Sub-agent, sandbox |

Khung này không có trục mà Manus nhấn mạnh nhất: thứ tự và độ ổn định của context ảnh hưởng trực tiếp đến tiền và độ trễ qua cache. Nên mình bắt đầu từ đó.

## 1. Bố trí prompt để giữ cache

Prompt caching khớp theo tiền tố: hai request có chung đoạn đầu giống hệt nhau thì phần tính toán cho đoạn đó dùng lại được, và sai khác một token ở vị trí k làm mất cache từ k trở đi. Cơ chế, bảng hệ số giá và một phép tính chi phí đầy đủ nằm ở bài [LLM dưới nắp máy](/bai-chia-se/llm-duoi-nap-may-cho-ky-su/). Ở đây mình chỉ giữ hai con số cần cho các quyết định bên dưới, theo [tài liệu prompt caching của Claude](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) lúc soạn bài: ghi cache tốn 1,25 lần giá đầu vào (thời hạn 5 phút) hoặc 2 lần (1 giờ), đọc cache tốn 0,1 lần (thấp hơn ở vài model mới).

Agent hưởng lợi nhiều vì mỗi lượt gửi lại toàn bộ lịch sử và chỉ thêm một ít ở đuôi. Bài của Manus cho biết tỉ lệ token đầu vào so với đầu ra của họ trung bình khoảng 100:1, và gọi tỉ lệ trúng KV-cache là chỉ số quan trọng nhất của một agent chạy production ([Context Engineering for AI Agents: Lessons from Building Manus](https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus)).

Để thấy bậc độ lớn, một phép tính giả định do mình đặt số: tiền tố cố định 10 nghìn token, mỗi lượt thêm 2 nghìn, chạy 50 lượt, không lần nào trượt cache, bỏ qua token đầu ra. Không cache, tổng đầu vào là 2.950 nghìn token. Có cache, quy đổi theo hệ số trên còn khoảng 419 nghìn, tức rẻ hơn khoảng 7 lần. Một thay đổi ở đầu prompt giữa chừng thì lượt đó trả giá ghi lại toàn bộ.

### Thứ tự bố trí

Theo tài liệu của Claude, tiền tố được ghép theo thứ tự tools, system, rồi messages, và thay đổi ở tầng nào làm mất cache của tầng đó cùng mọi tầng sau. Từ đó ra nguyên tắc: thứ ổn định nhất đứng trước, thứ biến động nhất đứng cuối.

```text
+----------------------------------+
| mô tả tool                       |  ổn định nhất: đổi là mất toàn bộ cache
+----------------------------------+
| system prompt                    |  không chứa giờ hiện tại, ID phiên
+----------------------------------+
| memory file, tài liệu nền        |  đổi theo dự án, không đổi theo lượt
+----------------------------------+  <- điểm đánh dấu cache
| lịch sử: chỉ nối thêm            |  không sửa lượt cũ
|   lượt 1, kết quả tool 1         |
|   lượt 2, kết quả tool 2 ...     |
+----------------------------------+  <- điểm đánh dấu cache dịch dần xuống
| thông tin biến động + lượt mới   |  ngày giờ, trạng thái tức thời
+----------------------------------+
```

Các quy tắc thực hành, mỗi dòng ghi nguồn:

- **Không đặt thứ biến động ở đầu** (Manus). Ví dụ của họ là timestamp chính xác đến giây ở đầu system prompt: mỗi request một giá trị, cache không bao giờ trúng.
- **Context chỉ nối thêm** (Manus). Không sửa hành động hay kết quả cũ.
- **Tuần tự hóa tất định** (Manus). Nhiều thư viện JSON không đảm bảo thứ tự khóa; cùng một object mà hai lần ra hai chuỗi khác nhau là trượt cache mà không ai nhận ra.
- **Biết cái gì phá cache ngoài văn bản** (tài liệu Claude). Đổi mô tả tool phá tất cả; đổi `tool_choice` hoặc thêm bớt ảnh phá phần messages.
- **Đo** (tài liệu Claude). Response trả về `cache_read_input_tokens` và `cache_creation_input_tokens`. Mình khuyên đưa tỉ lệ đọc trên tổng lên dashboard: nó tụt là có thứ gì ở đầu prompt đang đổi.

### Đừng thêm bớt tool giữa chừng

Mô tả tool nằm ở đầu tiền tố, nên thêm hoặc bớt một tool giữa phiên là mất cache toàn bộ. Manus nêu thêm lý do thứ hai: lịch sử còn nhắc đến một tool đã bị gỡ thì model bối rối, dễ gọi tool không tồn tại. Giải pháp của họ là giữ nguyên danh sách tool và chặn ở bước giải mã: một máy trạng thái che logit (điểm số model gán cho từng token ứng viên) để model chỉ chọn được nhóm tool hợp lệ ở trạng thái hiện tại. Để việc che đơn giản, họ đặt tên tool theo tiền tố chung như `browser_` hay `shell_`.

Cách này cần quyền can thiệp vào quá trình giải mã, hoặc API hỗ trợ điền sẵn phần đầu câu trả lời. Theo mình, nếu chỉ có API thông thường thì phương án gần nhất là tham số giới hạn lựa chọn tool; với Claude, đổi `tool_choice` vẫn làm mất cache phần messages, tức rẻ hơn đổi danh sách tool chứ không miễn phí. Một lối khác là nạp trễ định nghĩa tool, xem bài [Thiết kế tool cho agent](/bai-chia-se/thiet-ke-tool-mcp-skill-cho-agent/).

## 2. Nạp trước hay just-in-time

Có hai cách đưa kiến thức vào context. Nạp trước: truy xuất bằng embedding rồi chèn vào prompt trước khi model chạy. Just-in-time: chỉ giữ trong context các định danh nhẹ (đường dẫn file, câu truy vấn, link) và để agent tự dùng tool lấy nội dung khi cần.

Anthropic mô tả Claude Code như một mô hình lai: memory file được nạp sẵn từ đầu, còn code thì tìm lúc chạy bằng glob và grep. Cái giá họ ghi: khám phá lúc chạy chậm hơn, và thiếu chỉ dẫn tốt thì agent tiêu context vào ngõ cụt. Với nội dung ít thay đổi như tài liệu pháp lý hay tài chính, họ gợi ý nạp trước một phần rồi để agent tự tìm thêm.

Cách chọn mình rút ra: nội dung đổi nhanh và có cấu trúc để lần theo (code, hệ thống file) thì just-in-time. Kho văn bản lớn, phẳng, ít đổi, câu hỏi cần đáp nhanh thì nạp trước, và nhớ phát hiện của Chroma về distractor: năm đoạn "gần đúng" có thể tệ hơn một đoạn đúng. So sánh chi tiết giữa pipeline embedding và tìm kiếm kiểu agent ở bài [RAG và agentic search](/bai-chia-se/rag-va-agentic-search/).

## 3. Nén: compaction và tool result clearing

**Compaction** là khi context gần đầy, harness cho model tóm tắt lịch sử rồi chạy tiếp với bản tóm tắt. Anthropic mô tả Claude Code giữ lại các quyết định kiến trúc, lỗi chưa giải quyết, chi tiết triển khai, và bỏ kết quả tool trùng lặp. Lời khuyên tinh chỉnh của họ: tối ưu recall trước (đừng sót thứ cần), rồi mới đến precision (cắt thứ thừa).

Rủi ro cốt lõi là tóm tắt không đảo ngược được: không ai biết trước chi tiết nào ở lượt 12 sẽ thành mấu chốt ở lượt 80. Tài liệu Claude Code tự cảnh báo rằng chỉ dẫn chi tiết từ đầu hội thoại có thể mất sau compaction, và khuyên đặt quy tắc cần bền vào memory file.

**Tool result clearing** là dạng nén nhẹ hơn: thay kết quả tool cũ bằng một dòng giữ chỗ, giữ nguyên các lượt suy luận và lời gọi tool. Anthropic gọi đây là một trong những dạng nén an toàn nhất. Claude API có sẵn tính năng này ở dạng beta; mặc định nó kích hoạt khi prompt vượt 100 nghìn token đầu vào và giữ lại 3 lượt dùng tool gần nhất ([Context editing](https://platform.claude.com/docs/en/build-with-claude/context-editing)).

**Nén có phục hồi** là biến thể của Manus: được phép bỏ nội dung trang web khỏi context miễn là giữ URL, bỏ nội dung tài liệu miễn là giữ đường dẫn file trong sandbox. Thông tin không mất, chỉ chuyển ra ngoài, cần thì đọc lại.

### Nén đánh nhau với cache

Mọi thao tác sửa lịch sử đều đổi tiền tố, tức phá cache từ điểm sửa trở đi. Tài liệu context editing của Claude viết rõ việc xóa kết quả tool làm mất hiệu lực phần tiền tố đã cache, và vì thế có tham số `clear_at_least`: chỉ xóa khi xóa được ít nhất một lượng token đủ đáng với chi phí ghi lại cache.

Nguyên tắc mình suy ra: nén theo mẻ lớn và thưa, không nén lắt nhắt mỗi lượt.

## 4. Ghi chú có cấu trúc và bộ nhớ ngoài

Agent ghi tiến độ, quyết định và việc còn dở ra file, rồi đọc lại khi cần hoặc sau khi context bị làm mới. Ghi chú sống sót qua compaction và qua phiên mới. Anthropic dẫn ví dụ Claude chơi Pokémon: agent tự duy trì ghi chú về tiến độ luyện tập, bản đồ, chiến thuật qua hàng nghìn bước mà không cần được dặn về cấu trúc bộ nhớ.

Manus thêm một tác dụng thứ hai, họ gọi là recitation (tụng lại). Một tác vụ của Manus trung bình cần khoảng 50 tool call. Agent liên tục viết lại file `todo.md`, và việc viết lại đó đưa mục tiêu tổng thể xuống cuối context, vùng model chú ý nhiều.

Anthropic cho rằng ghi chú hợp với việc phát triển lặp có mốc rõ ràng. Phần còn lại của đoạn này là suy luận của mình từ bảng bốn kiểu hỏng. Ghi chú sai bị tin lâu hơn một lượt hội thoại sai, vì nó được đọc lại nhiều lần (poisoning). Ghi chú chỉ nối thêm thì tự nó thành gánh nặng context. Vì thế ghi chú nên có cấu trúc cố định (việc đã xong, việc đang làm, quyết định và lý do, câu hỏi mở) và được viết lại chứ không chỉ nối thêm.

Dùng file làm trạng thái cho agent chạy qua nhiều phiên là một thiết kế đầy đủ hơn, xem bài [Multi-agent và agent chạy dài](/bai-chia-se/multi-agent-va-agent-chay-dai/).

## 5. Sub-agent để cô lập context

Sub-agent nhận một việc, chạy trong context window riêng, và chỉ trả về bản cô đọng. Anthropic cho biết bản trả về thường khoảng 1.000 đến 2.000 token, trong khi sub-agent có thể đã dùng hàng chục nghìn token để khám phá. Toàn bộ phần khám phá đó không chạm vào context của agent chính.

Nhìn từ góc context, đây là kỹ thuật cô lập mạnh nhất và đắt nhất: sub-agent khởi đầu trắng nên chỉ biết những gì lời giao việc viết ra, và tổng token tăng nhiều lần. Con số chi phí, điều kiện để tách có lợi và mẫu lời giao việc ở bài [Multi-agent và agent chạy dài](/bai-chia-se/multi-agent-va-agent-chay-dai/).

## 6. Giữ lỗi lại, và tránh lịch sử rập khuôn

**Giữ lỗi trong context.** Phản xạ tự nhiên khi agent gọi tool sai là dọn dấu vết: thử lại âm thầm, xóa lượt lỗi. Manus khuyên ngược lại: khi model thấy hành động hỏng cùng thông báo lỗi, nó ngầm điều chỉnh và ít lặp lại hành động tương tự.

Quy tắc này có điểm dừng. Tài liệu Claude Code khuyên rằng sau hai lần sửa mà vẫn sai thì nên bắt đầu phiên mới với prompt tốt hơn, vì lần sửa thứ ba chỉ thêm vào một hội thoại đã chứa hai lần thử hỏng để model đọc đi đọc lại ([Best practices](https://code.claude.com/docs/en/best-practices)). Mình hiểu hai lời khuyên này bổ sung cho nhau: giữ lỗi của tool (thông tin về môi trường), nhưng đừng để các lần thử sai của chính model chồng lên nhau thành mẫu.

**Tránh few-shot rập khuôn.** Model bắt chước mẫu trong context. Manus nêu ví dụ duyệt một loạt 20 hồ sơ ứng viên: lịch sử gồm các cặp hành động và kết quả giống nhau về hình thức, đến giữa chừng agent trượt vào nhịp lặp, khái quát hóa quá đà hoặc bịa. Cách họ chữa là thêm biến thiên có kiểm soát: đổi mẫu tuần tự hóa, đổi cách diễn đạt, thêm nhiễu nhỏ về thứ tự và định dạng.

Lưu ý của mình: biến thiên đặt sai chỗ sẽ phá cache. Chỉ biến thiên ở phần được nối thêm sau cùng, và mỗi lượt đã ghi thì giữ nguyên.

## Chỗ các nguồn nói khác nhau

Hai cột giữa tóm quan điểm của nguồn. Cột "cách dung hòa" là đề xuất của mình, chưa có số liệu đỡ lưng.

| Chủ đề | Quan điểm A | Quan điểm B | Cách dung hòa |
|---|---|---|---|
| Sửa lịch sử | Anthropic: compaction và xóa kết quả tool là công cụ chính cho việc dài | Manus: context chỉ nối thêm, nén không đảo ngược là rủi ro | Nén theo mẻ lớn, thưa; ưu tiên dạng phục hồi được (giữ đường dẫn, URL) |
| Danh sách tool | LangChain viết rằng vài paper gần đây cho thấy dùng RAG trên mô tả tool cải thiện độ chính xác chọn tool khoảng 3 lần (bài không nêu chi tiết phép đo) | Manus: đừng thêm bớt tool giữa chừng, hãy che | Chọn tập tool một lần ở đầu phiên rồi cố định; cần hạn chế theo trạng thái thì che |
| Ví dụ trong prompt | Anthropic: một bộ ví dụ đa dạng, điển hình tốt hơn danh sách mọi trường hợp biên | Manus: đừng để bị few-shot | Không thật sự mâu thuẫn. Anthropic nói về ví dụ do người soạn, Manus nói về lịch sử hành động tự sinh. Điểm chung là đa dạng |
| Cấu trúc context | Anthropic: tổ chức prompt thành các phần rõ ràng | Chroma: văn bản nền mạch lạc cho kết quả kém hơn bản xáo trộn | Chưa dung hòa được. Chroma đo trên bài tìm đáp án trong văn bản nền, không đo chỉ dẫn. Đừng suy rộng thành "hãy xáo trộn context" |

Theo mình, bối cảnh giải thích một phần khác biệt. Manus xây trên model của bên khác và trả tiền theo token, nên cache là chuyện sống còn. Anthropic viết từ góc nhìn người làm model, ưu tiên chất lượng suy luận trên từng lượt.

## Checklist thiết kế

Bố trí và cache:

- Tiền tố (mô tả tool, system prompt) có byte nào đổi giữa các request không? Kiểm tra timestamp, ID ngẫu nhiên, thứ tự khóa JSON.
- Có theo dõi tỉ lệ token đọc từ cache trên tổng token đầu vào không?
- Danh sách tool có cố định trong suốt một phiên không?
- Khoảng nghỉ giữa các lượt (ví dụ chờ người duyệt) có vượt thời hạn cache không?

Nội dung:

- Mỗi kết quả tool có giới hạn kích thước không? Kết quả lớn có được ghi ra file và chỉ trả về đường dẫn kèm tóm tắt không?
- Kiến thức nền: cái gì nạp sẵn, cái gì để agent tự tìm? Phần nạp sẵn có đủ ngắn để đáng chỗ ở mọi lượt không?
- Có nguồn nào tạo distractor không: kết quả tìm kiếm trùng lặp, nhiều phiên bản của cùng một file?

Việc dài hơi:

- Ngưỡng compaction ở đâu, và bản tóm tắt bắt buộc giữ những gì (quyết định, lỗi đang mở, việc tiếp theo)?
- Agent có file ghi chú với cấu trúc cố định không? Mục tiêu có được nhắc lại ở cuối context không?
- Lỗi của tool có được giữ lại không? Có giới hạn số lần thử lại trước khi dừng hoặc làm mới phiên không?
- Việc lặp theo lô có nên chia thành nhiều phiên ngắn hoặc sub-agent thay vì một lịch sử dài đồng dạng?

## Điều còn mở, chưa chắc

- **Cơ chế của context rot.** Chroma đo được hiện tượng nhưng không giải thích được. "Attention budget" là hình dung hợp lý, chưa phải cơ chế đã kiểm chứng.
- **Model mới có làm phần này lỗi thời không.** Chroma thử các model giữa năm 2025. Tài liệu Claude Code lúc soạn bài viết rằng với context window 1 triệu token ở phần lớn model hiện hành, hiếm khi hết context, nhưng context gọn vẫn cho câu trả lời tốt hơn và rẻ hơn. Mình chưa thấy nghiên cứu độc lập cùng phương pháp với Chroma đo lại trên các model mới nhất, nên không nên giả định theo hướng nào. Lập luận về chi phí và độ trễ thì không phụ thuộc model.
- **Đo mất mát của compaction.** Chưa có phép đo chuẩn cho câu hỏi "bản tóm tắt này đã làm mất gì".
- **Các mẹo của Manus có tổng quát không.** Bài của họ là kinh nghiệm từ một sản phẩm, không kèm số liệu đối chứng. Chính tác giả gọi quá trình tìm ra chúng, nửa đùa, là "Stochastic Graduate Descent" và cho biết đã viết lại framework bốn lần. Nên xem là giả thuyết để thử trên hệ thống của bạn.
- **Giá cache thay đổi.** Hệ số giá và thời hạn cache trong bài lấy từ tài liệu lúc soạn (11/2026). Trước khi tối ưu, hãy đọc bảng giá hiện hành.

## Nguồn đọc thêm

- [Effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) (Anthropic, 9/2025): khung tổng thể, gồm attention budget, just-in-time, compaction, ghi chú, sub-agent. Nên đọc trước.
- [Context Engineering for AI Agents: Lessons from Building Manus](https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus) (Yichao "Peak" Ji, 7/2025): góc nhìn của người vận hành agent trên model của bên khác; phần KV-cache và che tool chi tiết nhất trong các nguồn mình đọc.
- [Context Rot](https://www.trychroma.com/research/context-rot) (Chroma, 7/2025): bằng chứng thực nghiệm trên 18 model. Đọc phần thiết kế thí nghiệm và phần giới hạn, đừng chỉ đọc kết luận.
- [Context Engineering](https://www.langchain.com/blog/context-engineering-for-agents) (LangChain, 7/2025): khung write, select, compress, isolate và bốn kiểu hỏng của context. Dùng làm mục lục để tra các nghiên cứu liên quan.
- [Prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) và [Context editing](https://platform.claude.com/docs/en/build-with-claude/context-editing) (tài liệu Claude API): thứ tự tiền tố, cái gì phá cache, hệ số giá, và tương tác giữa xóa kết quả tool với cache.
- [Best practices](https://code.claude.com/docs/en/best-practices) (tài liệu Claude Code): quy tắc làm mới phiên sau hai lần sửa hỏng và cách nhìn hiện hành về giới hạn context.
