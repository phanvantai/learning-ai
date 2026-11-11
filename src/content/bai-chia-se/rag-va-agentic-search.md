---
title: "RAG và agentic search: pipeline hỏng ở đâu, sửa bằng gì, khi nào bỏ hẳn index"
date: 2026-11-08
tags: [RAG, Agent]
description: Đi từng khâu của pipeline RAG cổ điển để thấy nó hỏng ở đâu, những cải tiến nào có số liệu đỡ lưng, vì sao coding agent chuộng grep hơn vector index, và cách đo retrieval để khỏi đoán mò. Có ghi chú riêng cho tiếng Việt.
loai: chuyen-sau
---

Câu hỏi "làm sao cho model biết dữ liệu của mình" hiện có ba câu trả lời chính: lập index rồi truy xuất trước khi hỏi (RAG), cho model tự đi tìm bằng tool (agentic search), hoặc nhét thẳng cả tài liệu vào context. Bài này đi qua cơ chế của từng cách, chỗ từng cách hỏng, và một bảng quyết định ở gần cuối. Con số nào đưa ra đều có nguồn, và mình ghi rõ nguồn nào là tự báo cáo, chỗ nào là suy luận của mình.

## Pipeline RAG cổ điển

RAG (retrieval augmented generation) tách việc trả lời thành hai pha: tìm đoạn văn bản liên quan, rồi đưa các đoạn đó vào prompt để model viết câu trả lời.

```text
LẬP INDEX (offline, chạy lại khi dữ liệu đổi)

  tài liệu --> chunking --> embedding model --> vector index
                   |
                   +------> (tuỳ chọn) index BM25

TRUY VẤN (online, mỗi câu hỏi)

  câu hỏi --> embedding --> tìm top-k vector gần nhất --+
     |                                                  +--> gộp --> rerank --> top-n
     +------> (tuỳ chọn) BM25 top-k --------------------+                        |
                                                                                 v
                                             prompt = hướng dẫn + top-n chunk + câu hỏi
                                                                                 |
                                                                                 v
                                                                    LLM sinh câu trả lời
```

Vài thuật ngữ dùng suốt bài:

- **Chunking**: cắt tài liệu thành đoạn nhỏ, thường vài trăm token.
- **Embedding**: model biến một đoạn văn thành vector, sao cho hai đoạn gần nghĩa thì vector gần nhau.
- **BM25**: thuật toán xếp hạng theo từ khóa, chấm điểm dựa trên tần suất từ trong đoạn và độ hiếm của từ trong toàn kho. Nó không hiểu nghĩa nhưng bắt chuẩn các chuỗi phải khớp chính xác.
- **Reranker**: model chấm lại độ liên quan của từng cặp (câu hỏi, chunk). Embedding mã hóa câu hỏi và chunk riêng rẽ. Reranker kiểu cross-encoder đọc cả hai cùng lúc nên chính xác hơn và chậm hơn, vì vậy chỉ dùng để lọc lại danh sách ứng viên đã thu hẹp.

Trong kiến trúc này, model sinh câu trả lời không tham gia vào việc tìm, nên tìm sai thì không có vòng nào để sửa.

## Bốn chỗ pipeline này hỏng

### 1. Chunk mất ngữ cảnh

Cắt tài liệu ra là cắt luôn những gì giúp hiểu đoạn đó. Bài Contextual Retrieval của Anthropic đưa ví dụ một chunk trong báo cáo tài chính chỉ ghi doanh thu tăng 3% so với quý trước: đứng riêng thì không biết công ty nào, quý nào. Embedding của chunk đó thiếu thông tin để khớp với câu hỏi "doanh thu quý 2/2023 của công ty X tăng bao nhiêu".

### 2. Truy vấn và tài liệu lệch từ vựng

Embedding khớp theo nghĩa nên chịu được cách diễn đạt khác nhau, nhưng lại yếu với thứ không có "nghĩa" để học: mã lỗi, mã sản phẩm, tên hàm, số hiệu văn bản. Bài Contextual Retrieval lấy ví dụ truy vấn chứa mã lỗi `TS-999`: embedding tìm ra nội dung về mã lỗi nói chung nhưng dễ trượt đúng mã đó, còn BM25 thì bắt được chuỗi chính xác. Điểm yếu của BM25 nằm ở chiều ngược lại: "xin nghỉ không lương" và "nghỉ việc riêng không hưởng lương" với nó là hai chuỗi ít liên quan.

Hai bên còn lệch nhau về hình thức: câu hỏi của người dùng ngắn, khẩu ngữ, còn tài liệu thì dài, trang trọng. Đây cũng là tiền đề của kỹ thuật HyDE ở phần sau.

### 3. Top-k nhiễu

Tìm kiếm vector luôn trả về k kết quả, kể cả khi trong kho không có gì liên quan. Model nhận k đoạn trông đúng chủ đề và dễ dùng chúng để trả lời. Tăng k để khỏi sót thì tốn token, và gặp hiện tượng mà paper Lost in the Middle (Liu và cộng sự, 2023) đo được trên bài toán hỏi đáp nhiều tài liệu: độ chính xác cao nhất khi thông tin liên quan nằm ở đầu hoặc cuối context, và giảm rõ khi nó nằm ở giữa. Paper đo trên các model năm 2023, nên đừng bê nguyên mức độ sang model hiện nay. Dù vậy tài liệu context window của Anthropic vẫn ghi rằng độ chính xác và khả năng nhớ lại giảm dần khi số token tăng (context rot, bài [Context engineering cho agent](/bai-chia-se/context-engineering-cho-agent/) nói kỹ hơn).

### 4. Câu hỏi nhiều bước

"Khách hàng nào có hợp đồng hết hạn trong quý này mà chưa có ai phụ trách gia hạn?" cần ít nhất hai lần tìm, lần sau phụ thuộc kết quả lần trước. Pipeline cổ điển chỉ tìm một lần với đúng câu hỏi gốc. Câu hỏi kiểu tổng hợp ("các điểm khác nhau giữa bản quy chế 2024 và 2025") cũng hỏng tương tự: câu trả lời không nằm trong chunk nào, phải so hai tập chunk mới ra.

## Cải tiến nào có bằng chứng

### Hybrid search: BM25 cộng embedding

Chạy cả hai rồi gộp kết quả. Cách gộp phổ biến là reciprocal rank fusion (RRF): mỗi chunk nhận điểm bằng tổng `1 / (k + thứ hạng)` qua các danh sách, với `k` là hằng số làm mượt (bài báo gốc của Cormack và cộng sự dùng 60). RRF chỉ dùng thứ hạng nên không phải lo chuẩn hóa hai thang điểm vốn không so sánh được với nhau.

```python
def rrf(ranked_lists, k=60):
    scores = {}
    for ranked in ranked_lists:            # mỗi list: id chunk theo thứ tự giảm dần
        for rank, chunk_id in enumerate(ranked, start=1):
            scores[chunk_id] = scores.get(chunk_id, 0.0) + 1.0 / (k + rank)
    return sorted(scores, key=scores.get, reverse=True)
```

Hybrid nhắm vào lỗi số 2: BM25 lo mã và thuật ngữ chính xác, embedding lo diễn đạt lại.

### Reranker

Lấy rộng (ví dụ top 100-150 từ hybrid), cho reranker chấm lại, giữ top-n nhỏ. Cách này giảm nhiễu ở lỗi số 3 mà không hy sinh recall, vì giai đoạn đầu được phép lấy thừa khi đã có giai đoạn sau lọc. Cái giá là thêm một lần gọi model cho mỗi truy vấn, phải tính vào ngân sách latency.

### Contextual retrieval

Ý tưởng của Anthropic (tháng 9/2024) nhắm vào lỗi số 1: trước khi embed và trước khi đưa vào index BM25, dùng một LLM rẻ viết thêm 50-100 token mô tả vị trí của chunk trong tài liệu gốc, rồi gắn vào đầu chunk. Prompt họ dùng yêu cầu model, khi được cho toàn văn tài liệu và một chunk, viết một đoạn ngữ cảnh ngắn gọn để định vị chunk đó trong tổng thể, phục vụ việc tìm kiếm.

Số liệu trong bài gốc, đo bằng tỷ lệ truy xuất hỏng (1 trừ recall@20, tức phần trăm tài liệu liên quan không lọt vào top 20 chunk):

| Cấu hình | Tỷ lệ hỏng top-20 | Giảm so với gốc |
| --- | --- | --- |
| Embedding thường (gốc) | 5,7% | |
| Contextual embedding | 3,7% | 35% |
| Contextual embedding + contextual BM25 | 2,9% | 49% |
| Như trên + reranker | 1,9% | 67% |

Vài điều khi đọc bảng này:

- Đây là số Anthropic tự báo cáo trên bộ dữ liệu của họ, với embedding model và reranker họ chọn. Mình đoán trên dữ liệu tiếng Việt thứ tự hơn kém vẫn giữ, nhưng chưa có số đo nào để khẳng định, và độ lớn thì chắc chắn phải tự đo.
- Ở bước rerank họ lấy top 150 rồi giữ lại top 20, và báo cáo rằng đưa 20 chunk cho model hiệu quả hơn 5 hay 10.
- Chi phí tạo ngữ cảnh một lần được họ ước tính 1,02 USD cho mỗi triệu token tài liệu, theo giá năm 2024 và giả định chunk 800 token, tài liệu 8.000 token, có dùng prompt caching (phần đầu prompt giống nhau giữa các lần gọi được tính giá rẻ): toàn văn tài liệu nằm ở phần đầu và được cache, mỗi chunk chỉ là phần đuôi thay đổi. Không cache thì mỗi chunk phải trả tiền đọc lại cả tài liệu.

### Query rewriting

Sửa câu hỏi trước khi tìm. Có mấy biến thể:

- Viết lại cho độc lập: trong hội thoại, câu "thế còn bản 2025?" vô nghĩa nếu đem đi tìm nguyên văn. Cho LLM viết lại thành câu hỏi đầy đủ dựa trên lịch sử chat. Theo mình, chatbot nhiều lượt gần như bắt buộc phải có bước này.
- Tách câu hỏi: câu nhiều bước thành nhiều truy vấn con, tìm riêng rồi gộp.
- HyDE (Gao và cộng sự, 2022): cho LLM viết một câu trả lời giả định rồi embed đoạn đó thay cho câu hỏi. Các tác giả lý giải rằng câu trả lời giả định có hình thức giống tài liệu hơn là giống câu hỏi, nên rút ngắn khoảng cách hình thức ở lỗi số 2. Paper báo cáo vượt rõ baseline không giám sát (Contriever). Theo suy luận của mình, rủi ro là khi model bịa sai hướng trong đoạn giả định thì truy vấn bị kéo lệch theo.

## Agentic search: để model tự đi tìm

Thay vì tìm một lần rồi đưa kết quả cho model, ta đưa cho model vài tool tìm kiếm và để nó chạy trong vòng lặp: tìm, đọc kết quả, quyết định tìm tiếp cái gì, đến khi đủ thông tin.

```python
tools = [grep, glob, read_file]          # tìm theo regex, liệt kê file, đọc một khoảng dòng
messages = [question]

while True:
    reply = llm(system_prompt, tools, messages)
    messages.append(reply)
    if not reply.tool_calls:
        break                            # model thấy đủ, trả lời
    for call in reply.tool_calls:
        result = run(call)               # ví dụ: grep -n "retry_policy" src/
        messages.append(truncate(result))  # luôn giới hạn kích thước kết quả
```

Khác với pipeline ở trên, vòng lặp này có chỗ sửa sai. Tìm `retry_policy` không ra thì thử `RetryPolicy`, `backoff`, hoặc đọc file config để biết tên đúng. Lỗi số 2 (lệch từ vựng) được xử lý bằng cách thử lại, điều mà pipeline một lượt không làm được. Câu hỏi nhiều bước cũng xử lý được, vì kết quả lượt trước quyết định truy vấn lượt sau.

Model còn tự chọn đọc bao nhiêu. Thấy một dòng grep đáng ngờ thì nó mở 80 dòng xung quanh, không bị giới hạn trong ranh giới chunk cắt sẵn, nên lỗi số 1 nhẹ đi nhiều.

Bài Effective context engineering của Anthropic gọi hướng này là "just in time": agent giữ các tham chiếu nhẹ (đường dẫn file, truy vấn đã lưu, link) và chỉ nạp dữ liệu vào context lúc cần.

### Vì sao coding agent chuộng cách này

Trong một buổi phỏng vấn trên Latent Space (tháng 5/2025), Boris Cherny (người tạo Claude Code) kể rằng các bản đầu có thử RAG với index dựng sẵn, sau đó chuyển hẳn sang agentic search bằng glob và grep. Lý do ông nêu:

- Chất lượng: theo lời ông, nó vượt mọi phương án khác với khoảng cách lớn. Ông cũng nói thẳng rằng đánh giá này chủ yếu dựa trên cảm nhận nội bộ ("vibes"), không phải một thí nghiệm công bố.
- Index bị cũ: code đổi liên tục, index phải đồng bộ lại, và lệch pha là nguồn lỗi khó thấy.
- Bảo mật: index phải nằm ở đâu đó, và đẩy codebase nhạy cảm lên dịch vụ bên thứ ba là rủi ro.

Theo ông, tránh được các vấn đề của index thì phải trả bằng latency và token.

Còn một lý do nằm ở bản chất dữ liệu (suy luận của mình, buổi phỏng vấn không nêu): code là loại văn bản mà định danh (tên hàm, tên biến, đường dẫn import) vừa chính xác vừa duy nhất, nên tìm theo chuỗi hiệu quả, và cấu trúc thư mục tự nó đã là một dạng index. Văn bản tự nhiên không có tính chất đó.

### Kết quả ngược chiều từ Cursor

Cursor công bố kết quả ngược chiều: khi thêm semantic search (index embedding) bên cạnh grep cho agent, độ chính xác trả lời câu hỏi về codebase cao hơn trung bình 12,5% trên bộ đánh giá nội bộ của họ (từ 6,5% đến 23,5% tuỳ model; bài không nói rõ đây là mức tăng tương đối hay điểm phần trăm). Trong A/B test trực tuyến, tỷ lệ code do agent viết được giữ lại tăng 0,3%, và tăng 2,6% với codebase từ 1.000 file trở lên. Kết luận của họ là kết hợp grep với semantic search cho kết quả tốt nhất.

Cả hai nguồn đều là tự báo cáo trên hệ thống của chính mình, và đọc kỹ thì không hẳn mâu thuẫn:

- Cursor không nói grep kém, họ nói grep một mình kém hơn grep cộng semantic, rõ nhất trên codebase lớn.
- Semantic search trong thí nghiệm của Cursor là một tool cho agent gọi trong vòng lặp, không phải pipeline một lượt. Hai bên vì thế đều đang dùng tìm kiếm nhiều vòng có phản hồi, chỉ khác bộ tool bên trong.
- Bài Effective context engineering cũng mô tả hướng lai: nạp sẵn file hướng dẫn dự án vào context, phần còn lại để agent tự tìm bằng glob và grep.

Cách đọc của mình (chưa nguồn nào kiểm chứng trực tiếp): vòng lặp là thứ tạo khác biệt lớn, còn dùng tool tìm kiếm nào bên trong vòng lặp thì tuỳ dữ liệu và quy mô.

### Chỗ agentic search hỏng

Danh sách này là phân tích của mình từ cơ chế vòng lặp, không trích từ một nguồn cụ thể.

- Chậm và tốn token. Mỗi vòng là một lần gọi model và phải gửi lại toàn bộ lịch sử. Câu hỏi cần 8 vòng tìm là 8 lần gọi, so với một lần truy xuất index rồi một lần gọi.
- Kết quả tool làm phình context. Một lệnh grep vụng trả về hàng nghìn dòng. Tool phải cắt bớt và báo rõ là đã cắt để model thu hẹp truy vấn (xem [Thiết kế tool, MCP và Skill cho agent](/bai-chia-se/thiet-ke-tool-mcp-skill-cho-agent/)).
- Dừng sớm hoặc không chịu dừng. Model có thể dừng ở kết quả đầu tiên trông hợp lý, hoặc lòng vòng tìm thứ không tồn tại. Cần giới hạn số vòng và hướng dẫn khi nào được kết luận "không tìm thấy".
- Không có từ khóa để bám. Câu hỏi khái niệm trên kho văn bản tự nhiên ("các trường hợp công ty từng xử lý khách phàn nàn về giao hàng trễ") khó diễn đạt thành regex. Đây là chỗ một tool tìm theo nghĩa phát huy tác dụng.
- Quy mô. Grep trên vài nghìn file là tức thì, còn trên hàng triệu tài liệu thì cần index, dù là index từ khóa hay vector.

## Phương án thứ ba: nhét thẳng vào context

Bài Contextual Retrieval mở đầu bằng một lời khuyên dễ bị bỏ qua: nếu kho kiến thức nhỏ hơn khoảng 200.000 token (họ ước lượng cỡ 500 trang), hãy đưa toàn bộ vào prompt và khỏi cần RAG. Kết hợp prompt caching thì phần tài liệu cố định ở đầu prompt được tính giá đọc cache ở các lần gọi sau, với Claude API là khoảng 0,1 lần giá input thường hoặc thấp hơn tuỳ model. Cơ chế và phép tính chi tiết có trong bài [LLM dưới nắp máy](/bai-chia-se/llm-duoi-nap-may-cho-ky-su/).

Cách này cần tài liệu đặt ở đầu prompt và giống nhau từng byte giữa các lần gọi. Lưu lượng phải đủ đều, vì cache có thời hạn (Claude API mặc định 5 phút, làm mới mỗi lần đọc trúng) nên hỏi rải rác vài câu mỗi ngày thì lần nào cũng trả giá ghi. Tài liệu cũng phải ít đổi.

Context rot vẫn áp dụng ở đây, nên vừa context window chưa chắc là model dùng tốt hết. Với câu hỏi cần đúng một chi tiết nhỏ trong vài trăm trang, nên đo so sánh với một pipeline retrieval trước khi tin.

## Chọn cái nào

Bảng dưới là tổng hợp của mình từ các phần trên, dùng làm điểm xuất phát chứ không thay cho việc đo.

| Tình huống | Hướng nên thử trước | Lý do |
| --- | --- | --- |
| Kho nhỏ (dưới vài trăm nghìn token), ít đổi, hỏi thường xuyên | Nhét cả vào context + prompt caching | Không có khâu retrieval để hỏng; chi phí được cache gánh |
| Codebase, thư mục tài liệu có cấu trúc, dữ liệu đổi liên tục | Agentic search với grep/glob/đọc file | Định danh chính xác, không lo index cũ, có vòng sửa sai |
| Kho lớn văn bản tự nhiên, cần trả lời nhanh, câu hỏi một bước | Hybrid (BM25 + embedding) + reranker | Latency thấp và ổn định, chi phí mỗi truy vấn dự đoán được |
| Như trên nhưng chunk hay thiếu ngữ cảnh (hợp đồng, báo cáo, quy chế) | Thêm contextual retrieval | Nhắm đúng lỗi chunk mất ngữ cảnh, trả phí một lần lúc lập index |
| Kho lớn, câu hỏi nhiều bước hoặc cần tổng hợp | Agent với tool tìm kiếm hybrid | Vòng lặp lo phần nhiều bước, index lo phần quy mô |
| Dữ liệu có phân quyền theo người dùng | Lọc quyền ngay trong tool hoặc truy vấn, trước khi kết quả đến model | Model không phải là lớp kiểm soát truy cập |

Thứ tự mình sẽ thử khi bắt đầu một dự án: phương án đơn giản nhất còn khả thi (nhét context, rồi agentic search trên file), dựng bộ eval, và chỉ thêm index khi số đo cho thấy cần.

## Ghi chú cho tiếng Việt

Toàn bộ mục này là suy luận từ cơ chế và từ tài liệu của công cụ. Mình chưa tìm được benchmark retrieval tiếng Việt nào đủ tin cậy để trích, nên hãy coi mỗi ý dưới đây là giả thuyết cần đo trên dữ liệu của bạn.

Tách từ ảnh hưởng trực tiếp đến BM25. Tiếng Việt viết cách nhau theo âm tiết, không theo từ. Một bộ phân tích mặc định tách theo khoảng trắng sẽ coi "học sinh" là hai token "học" và "sinh". Với BM25 kiểu túi từ, "học sinh" và "sinh học" vì thế có cùng tập token, và từ "sinh" trong "sinh viên", "vệ sinh", "sinh nhật" đều góp điểm cho truy vấn "sinh học". Có hai hướng xử lý:

- Tách từ trước khi index bằng thư viện như underthesea (hàm `word_tokenize` của nó gộp "Chàng trai", "Quảng Trị", "khởi nghiệp" thành từng từ), và áp dụng đúng cùng một bộ tách cho truy vấn. Rủi ro là bộ tách từ cũng sai, nhất là với tên riêng và thuật ngữ chuyên ngành, và sai lệch giữa lúc index với lúc truy vấn sẽ làm mất kết quả.
- Giữ token âm tiết nhưng index thêm cặp âm tiết liền nhau (bigram/shingle), để "học sinh" đúng thứ tự được điểm cao hơn. Đơn giản hơn, index to hơn.

Chưa có số đo nào cho mình biết hướng nào thắng. Hãy đo bằng recall@k trên dữ liệu của bạn.

Dấu và chuẩn hóa Unicode là vấn đề tiếp theo. Người dùng hay gõ không dấu ("hoa don dien tu"). BM25 khớp theo chuỗi nên truy vấn không dấu sẽ không khớp tài liệu có dấu, trừ khi bạn index thêm một trường đã bỏ dấu và tìm trên cả hai. Ngoài ra, cùng một chữ có dấu có thể được mã hóa dạng dựng sẵn (NFC) hoặc dạng tổ hợp (NFD), nhìn giống hệt nhau nhưng khác byte. Chuẩn hóa về một dạng (thường là NFC) ở cả khâu nạp tài liệu và khâu truy vấn, nếu không thì khớp chuỗi và grep sẽ trượt mà không rõ vì sao.

Embedding và reranker nên là model đa ngữ. Model huấn luyện chủ yếu trên tiếng Anh nhiều khả năng cho vector kém với tiếng Việt. BGE-M3 là một lựa chọn mã nguồn mở hay được nhắc: trang model ghi hỗ trợ hơn 100 ngôn ngữ, đầu vào tới 8.192 token, và sinh được cả vector dense lẫn sparse (chính trang đó cũng khuyến nghị hybrid cộng rerank). Mình không khẳng định model nào tốt nhất cho tiếng Việt. Hãy lấy vài chục câu hỏi thật của hệ thống bạn và đo. Nhớ kiểm tra riêng trường hợp truy vấn không dấu và truy vấn trộn tiếng Anh.

Agentic search với grep chịu ảnh hưởng của cả chuyện dấu lẫn chuyện Unicode: model có thể grep "hóa đơn" trong khi file ghi "hoá đơn" (hai cách đặt dấu đều phổ biến), hoặc file ở dạng NFD. Nên chuẩn hóa dữ liệu lúc nạp, và ghi các biến thể chính tả vào hướng dẫn cho agent.

## Đo retrieval: tách hai khâu ra

Chỉ nhìn câu trả lời cuối thì không biết sửa ở đâu. Câu trả lời sai có thể vì không tìm ra đoạn đúng, hoặc tìm ra rồi nhưng model dùng sai. Hai trường hợp sửa ở hai chỗ khác nhau, nên phải đo riêng. Cách dựng bộ task và grader nói chung có trong bài [Eval cho ứng dụng LLM và agent](/bai-chia-se/eval-cho-agent/).

### Đo retrieval

Cần một bộ câu hỏi kèm danh sách chunk (hoặc tài liệu) được coi là đúng. Vài chục đến một trăm câu lấy từ câu hỏi thật là đủ để bắt đầu. Nhớ gồm cả câu hỏi mà kho không có đáp án.

- **recall@k**: trong số chunk đúng, bao nhiêu phần trăm lọt vào top k. Đây là chỉ số quan trọng nhất cho khâu lấy ứng viên, vì thứ không lọt vào top k thì các khâu sau không cứu được. Con số "tỷ lệ hỏng" của Anthropic ở trên là 1 trừ recall@20.
- **precision@k**: trong top k có bao nhiêu phần trăm liên quan. Đo mức nhiễu đưa cho model.
- **MRR, nDCG**: có tính đến thứ hạng, dùng khi so sánh reranker.

```python
def recall_at_k(retrieved_ids, relevant_ids, k):
    if not relevant_ids:                 # câu hỏi mà kho không có đáp án
        return None                      # chấm riêng: model có nói "không có" không
    hit = len(set(retrieved_ids[:k]) & set(relevant_ids))
    return hit / len(relevant_ids)
```

### Đo generation

Giữ cố định context (lý tưởng là context chứa sẵn đoạn đúng) rồi chấm câu trả lời theo hai trục:

- **Bám nguồn**: mọi khẳng định trong câu trả lời có nằm trong context được cấp không.
- **Đúng**: câu trả lời có khớp đáp án tham chiếu không.

Một hệ thống có thể bám nguồn tuyệt đối mà vẫn sai, khi nguồn được lấy về là sai.

### Đọc kết quả

| recall@k | Câu trả lời | Nguyên nhân có khả năng | Sửa ở đâu |
| --- | --- | --- | --- |
| Thấp | Sai | Không tìm ra | Chunking, hybrid, rewriting, embedding model |
| Cao | Sai | Tìm ra nhưng dùng sai hoặc bị nhiễu lấn | Rerank, giảm top-n, prompt, model |
| Thấp | Đúng | Model trả lời từ kiến thức huấn luyện | Nguy hiểm: sẽ sai khi dữ liệu của bạn khác kiến thức chung |
| Cao | Đúng | Ổn | |

### Với agentic search

Không có "top-k" cố định, nên đo theo quỹ đạo: agent có mở đúng file hoặc đúng đoạn chứa đáp án không, mất bao nhiêu vòng, tốn bao nhiêu token và thời gian cho mỗi câu hỏi. Token và thời gian là cái giá của phương pháp nên phải báo cáo cùng độ chính xác. Kết quả giữa các lần chạy cũng dao động hơn pipeline cố định, nên mỗi câu nên chạy vài lần.

## Checklist thiết kế

- Có bộ câu hỏi kèm nhãn để đo recall@k trước khi tối ưu bất cứ thứ gì không?
- Truy vấn có chứa mã, tên riêng, số hiệu không? Có thì cần BM25 hoặc grep, embedding một mình không đủ.
- Hệ thống làm gì khi không tìm thấy gì liên quan? Có ngưỡng điểm hoặc hướng dẫn để model nói "không có trong tài liệu" không?
- Phân quyền được áp ở tầng truy vấn, trước khi dữ liệu đến model chưa?
- Với tiếng Việt: đã chuẩn hóa Unicode, xử lý truy vấn không dấu, và thống nhất bộ tách từ giữa index với truy vấn chưa?
- Với agent: kết quả tool có bị giới hạn kích thước không, có giới hạn số vòng không?

## Điều còn mở / chưa chắc

- Index vector có còn cần cho code không. Claude Code và Cursor đưa ra hai câu trả lời khác nhau, đều dựa trên đánh giá nội bộ. Chưa có so sánh độc lập nào mình biết mà kiểm soát được cả model, harness lẫn quy mô codebase.
- Context dài đến đâu thì retrieval hết cần. Nhiều model hiện có context window 1 triệu token, nhưng context rot vẫn được chính tài liệu của nhà cung cấp ghi nhận và mức độ thay đổi theo từng thế hệ model. Ngưỡng 200.000 token trong bài của Anthropic là của năm 2024, mình không chắc con số hợp lý hiện nay là bao nhiêu.
- Contextual retrieval trên tiếng Việt. Cơ chế không phụ thuộc ngôn ngữ, nhưng mình chưa thấy số đo công khai nào cho tiếng Việt.
- Tách từ hay bigram âm tiết cho BM25 tiếng Việt. Mình nêu cả hai vì không có căn cứ để chọn hộ bạn.

## Nguồn đọc thêm

- [Bài giới thiệu Contextual Retrieval (Anthropic, 9/2024)](https://www.anthropic.com/news/contextual-retrieval): nguồn của bảng số liệu ở trên, kèm prompt tạo ngữ cảnh và lời khuyên bỏ RAG khi kho nhỏ.
- [Effective context engineering for AI agents (Anthropic)](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents): lập luận cho hướng "just in time" và hướng lai, cùng khái niệm context rot.
- [Claude Code: Anthropic's Agent in Your Terminal (Latent Space, 5/2025)](https://www.latent.space/p/claude-code): phần Boris Cherny giải thích vì sao bỏ RAG, đọc để thấy nhận định đó chắc chắn tới đâu.
- [Improving agent with semantic search (Cursor, 11/2025)](https://cursor.com/blog/semsearch): quan điểm ngược lại, có số liệu A/B, nên đọc cùng nguồn trên.
- [Lost in the Middle (Liu và cộng sự)](https://arxiv.org/abs/2307.03172): thí nghiệm gốc về việc vị trí thông tin trong context ảnh hưởng độ chính xác.
- [Precise Zero-Shot Dense Retrieval without Relevance Labels (HyDE)](https://arxiv.org/abs/2212.10496): paper gốc của kỹ thuật embed câu trả lời giả định.
- [BAAI/bge-m3 trên Hugging Face](https://huggingface.co/BAAI/bge-m3): mô tả một embedding model đa ngữ có cả dense và sparse, điểm xuất phát tốt để thử với tiếng Việt.
- [underthesea](https://github.com/undertheseanlp/underthesea): thư viện xử lý tiếng Việt có tách từ, dùng khi thử hướng tách từ cho BM25.
- [Prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) và [Context windows](https://platform.claude.com/docs/en/build-with-claude/context-windows) (tài liệu Claude API): giá và thời hạn cache, và chỗ tài liệu chính thức nói về context rot.
- Cormack, Clarke, Büttcher (2009), "Reciprocal Rank Fusion outperforms Condorcet and individual rank learning methods": bài báo gốc của RRF (mình không kiểm tra được link trong lúc viết nên chỉ ghi tên).
