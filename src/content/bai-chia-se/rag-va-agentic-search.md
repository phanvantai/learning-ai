---
title: "RAG và agentic search: pipeline hỏng ở đâu, sửa bằng gì, khi nào bỏ hẳn index"
date: 2026-11-08
tags: [RAG, Agent]
description: Đi từng khâu của pipeline RAG cổ điển để thấy nó hỏng ở đâu, những cải tiến nào có số liệu đỡ lưng, vì sao coding agent chuộng grep hơn vector index, và cách đo retrieval để khỏi đoán mò. Có ghi chú riêng cho tiếng Việt.
loai: chuyen-sau
---

Câu hỏi "làm sao cho model biết dữ liệu của mình" hiện có ba câu trả lời chính: lập index rồi truy xuất trước khi hỏi (RAG), cho model tự đi tìm bằng tool (agentic search), hoặc nhét thẳng cả tài liệu vào context. Bài này đi qua cơ chế của từng cách, chỗ từng cách hỏng, và một bảng quyết định ở gần cuối. Mình cố gắng chỉ đưa con số nào có nguồn, và ghi rõ nguồn nào là tự báo cáo.

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

- **Chunking**: cắt tài liệu thành đoạn nhỏ (thường vài trăm token) vì embedding của một đoạn ngắn biểu diễn nghĩa tập trung hơn embedding của cả tài liệu.
- **Embedding**: model biến một đoạn văn thành vector, sao cho hai đoạn gần nghĩa thì vector gần nhau.
- **BM25**: thuật toán xếp hạng theo từ khóa, chấm điểm dựa trên tần suất từ trong đoạn và độ hiếm của từ trong toàn kho. Nó không hiểu nghĩa nhưng bắt rất chuẩn các chuỗi phải khớp chính xác.
- **Reranker**: model chấm lại độ liên quan của từng cặp (câu hỏi, chunk). Khác với embedding (mã hóa câu hỏi và chunk riêng rẽ), reranker kiểu cross-encoder đọc cả hai cùng lúc nên chính xác hơn và chậm hơn, vì vậy chỉ dùng để lọc lại một danh sách ứng viên đã thu hẹp.

Điểm cần nhớ về kiến trúc: model sinh câu trả lời không tham gia vào việc tìm. Nó nhận top-n chunk như một sự đã rồi. Tìm sai thì không có vòng nào để sửa.

## Bốn chỗ pipeline này hỏng

### 1. Chunk mất ngữ cảnh

Cắt tài liệu ra là cắt luôn những gì giúp hiểu đoạn đó. Bài Contextual Retrieval của Anthropic đưa ví dụ một chunk trong báo cáo tài chính chỉ ghi doanh thu tăng 3% so với quý trước: đứng riêng thì không biết công ty nào, quý nào. Embedding của chunk đó gần như không mang thông tin để khớp với câu hỏi "doanh thu quý 2/2023 của công ty X tăng bao nhiêu".

### 2. Truy vấn và tài liệu lệch từ vựng

Embedding khớp theo nghĩa nên chịu được cách diễn đạt khác nhau, nhưng lại yếu với thứ không có "nghĩa" để học: mã lỗi, mã sản phẩm, tên hàm, số hiệu văn bản. Hai mã `TS-999` và `TS-998` gần như trùng nhau trong không gian vector, trong khi người hỏi cần đúng một cái. BM25 thì ngược lại: bắt đúng `TS-999`, nhưng "xin nghỉ không lương" và "nghỉ việc riêng không hưởng lương" với nó là hai chuỗi ít liên quan.

Lệch còn đến từ hình thức: câu hỏi của người dùng ngắn, khẩu ngữ, trong khi tài liệu dài, trang trọng. Hai thứ đó không nằm gần nhau trong không gian embedding như ta mong.

### 3. Top-k nhiễu

Tìm kiếm vector luôn trả về k kết quả, kể cả khi trong kho không có gì liên quan. Model nhận k đoạn "trông có vẻ đúng chủ đề" và có xu hướng dùng chúng. Tăng k để khỏi sót thì gặp hai vấn đề: chi phí token, và hiện tượng mà paper Lost in the Middle (Liu và cộng sự, 2023) đo được trên bài toán hỏi đáp nhiều tài liệu: độ chính xác cao nhất khi thông tin liên quan nằm ở đầu hoặc cuối context, và giảm rõ khi nó nằm ở giữa. Các model hiện nay đã khá hơn thời điểm paper đó, nhưng tài liệu context window của Anthropic vẫn ghi nhận rằng độ chính xác và khả năng nhớ lại giảm dần khi số token tăng (họ gọi là context rot).

### 4. Câu hỏi nhiều bước

"Khách hàng nào có hợp đồng hết hạn trong quý này mà chưa có ai phụ trách gia hạn?" cần ít nhất hai lần tìm, lần sau phụ thuộc kết quả lần trước. Pipeline cổ điển chỉ tìm một lần với đúng câu hỏi gốc. Câu hỏi kiểu tổng hợp ("các điểm khác nhau giữa bản quy chế 2024 và 2025") cũng hỏng theo cách tương tự: câu trả lời không nằm trong chunk nào cả, nó nằm ở hiệu của hai tập chunk.

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

Hybrid xử lý trực tiếp lỗi số 2: BM25 lo mã và thuật ngữ chính xác, embedding lo diễn đạt lại.

### Reranker

Lấy rộng (ví dụ top 100-150 từ hybrid), cho reranker chấm lại, giữ top-n nhỏ. Đây là cách giảm nhiễu ở lỗi số 3 mà không phải hy sinh recall: giai đoạn đầu được phép lấy thừa vì có giai đoạn sau lọc. Cái giá là thêm một lần gọi model cho mỗi truy vấn, nên phải tính vào ngân sách latency.

### Contextual retrieval

Ý tưởng của Anthropic (tháng 9/2024) nhắm vào lỗi số 1: trước khi embed và trước khi đưa vào index BM25, dùng một LLM rẻ viết thêm 50-100 token mô tả vị trí của chunk trong tài liệu gốc, rồi gắn vào đầu chunk. Prompt họ dùng yêu cầu model, khi được cho toàn văn tài liệu và một chunk, viết một đoạn ngữ cảnh ngắn gọn để định vị chunk đó trong tổng thể, phục vụ việc tìm kiếm.

Số liệu trong bài gốc, đo bằng tỷ lệ truy xuất hỏng (1 trừ recall@20, tức phần trăm tài liệu liên quan không lọt vào top 20 chunk):

| Cấu hình | Tỷ lệ hỏng top-20 | Giảm so với gốc |
| --- | --- | --- |
| Embedding thường (gốc) | 5,7% | |
| Contextual embedding | 3,7% | 35% |
| Contextual embedding + contextual BM25 | 2,9% | 49% |
| Như trên + reranker | 1,9% | 67% |

Vài chi tiết đáng lưu ý khi đọc bảng này:

- Đây là số trên bộ dữ liệu của chính Anthropic, tiếng Anh, với embedding model và reranker họ chọn. Trên dữ liệu tiếng Việt của bạn, thứ tự hơn kém nhiều khả năng giữ nguyên nhưng độ lớn thì phải tự đo.
- Ở bước rerank họ lấy top 150 rồi giữ lại top 20, và báo cáo rằng đưa 20 chunk cho model hiệu quả hơn 5 hay 10.
- Chi phí tạo ngữ cảnh một lần được họ ước tính 1,02 USD cho mỗi triệu token tài liệu, với điều kiện dùng prompt caching (lưu lại phần đầu prompt để các lần gọi sau có cùng phần đầu được tính giá rẻ): toàn văn tài liệu nằm ở phần đầu và được cache, mỗi chunk chỉ là phần đuôi thay đổi. Không cache thì mỗi chunk phải trả tiền đọc lại cả tài liệu.

### Query rewriting

Sửa câu hỏi trước khi tìm. Ba biến thể chính:

- **Viết lại cho độc lập**: trong hội thoại, câu "thế còn bản 2025?" vô nghĩa nếu đem đi tìm nguyên văn. Cho LLM viết lại thành câu hỏi đầy đủ dựa trên lịch sử chat. Đây là biến thể mình cho là gần như bắt buộc với chatbot nhiều lượt.
- **Tách câu hỏi**: câu nhiều bước thành nhiều truy vấn con, tìm riêng rồi gộp.
- **HyDE** (Gao và cộng sự, 2022): cho LLM viết một câu trả lời giả định rồi embed đoạn đó thay cho câu hỏi. Lý do nó hoạt động: câu trả lời giả định có hình thức giống tài liệu hơn là giống câu hỏi, nên rút ngắn khoảng cách hình thức ở lỗi số 2. Paper báo cáo vượt rõ baseline không giám sát (Contriever). Rủi ro: khi model bịa sai hướng trong đoạn giả định, truy vấn bị kéo lệch theo.

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

So với pipeline ở trên, khác biệt nằm ở ba điểm:

1. **Có vòng sửa sai.** Tìm `retry_policy` không ra thì thử `RetryPolicy`, `backoff`, hoặc đọc file config để biết tên đúng. Lỗi số 2 (lệch từ vựng) được xử lý bằng cách thử lại, điều mà pipeline một lượt không làm được.
2. **Xử lý được câu hỏi nhiều bước.** Kết quả lượt trước quyết định truy vấn lượt sau.
3. **Model tự chọn đọc bao nhiêu.** Thấy một dòng grep đáng ngờ thì mở 80 dòng xung quanh, thay vì bị giới hạn trong ranh giới chunk cắt sẵn. Lỗi số 1 gần như biến mất vì không còn chunk.

Bài Effective context engineering của Anthropic gọi hướng này là "just in time": agent giữ các tham chiếu nhẹ (đường dẫn file, truy vấn đã lưu, link) và chỉ nạp dữ liệu vào context lúc cần, giống cách con người dựa vào cấu trúc thư mục và tên file thay vì thuộc lòng nội dung.

### Vì sao coding agent chuộng cách này

Trong một buổi phỏng vấn trên Latent Space, Boris Cherny (người tạo Claude Code) kể rằng các bản đầu có thử RAG với index dựng sẵn, sau đó chuyển hẳn sang agentic search bằng glob và grep. Lý do ông nêu:

- Chất lượng: theo lời ông, nó vượt mọi phương án khác với khoảng cách lớn. Ông cũng nói thẳng rằng đánh giá này chủ yếu dựa trên cảm nhận nội bộ, kèm một ít benchmark nội bộ.
- Index bị cũ: code đổi liên tục, index phải đồng bộ lại, và lệch pha là nguồn lỗi khó thấy.
- Bảo mật: index phải nằm ở đâu đó, và đẩy codebase nhạy cảm lên dịch vụ bên thứ ba là rủi ro.
- Đơn giản vận hành: không có index thì không phải nuôi index.

Ông tóm tắt đánh đổi là trả bằng latency và token để có tìm kiếm tốt mà không kèm rủi ro bảo mật.

Mình nghĩ còn một lý do về bản chất dữ liệu: code là loại văn bản mà định danh (tên hàm, tên biến, đường dẫn import) vừa chính xác vừa duy nhất, nên tìm theo chuỗi rất hiệu quả, và cấu trúc thư mục tự nó đã là một dạng index. Văn bản tự nhiên không có tính chất đó.

### Nhưng không phải ai cũng đồng ý

Cursor công bố kết quả ngược chiều: khi thêm semantic search (index embedding) bên cạnh grep cho agent, độ chính xác trả lời câu hỏi về codebase tăng trung bình 12,5% trên bộ đánh giá nội bộ của họ (từ 6,5% đến 23,5% tuỳ model). Trong A/B test trực tuyến, tỷ lệ code do agent viết được giữ lại tăng 0,3%, và tăng 2,6% với codebase từ 1.000 file trở lên. Kết luận của họ là kết hợp grep với semantic search cho kết quả tốt nhất.

Hai nguồn này không hẳn mâu thuẫn, và cả hai đều là tự báo cáo trên hệ thống của chính mình:

- Cursor không nói grep kém, họ nói grep một mình kém hơn grep cộng semantic, rõ nhất trên codebase lớn.
- Semantic search trong thí nghiệm của Cursor là một tool cho agent gọi trong vòng lặp, không phải pipeline một lượt. Tức là ranh giới thật không nằm giữa "vector" và "grep", mà giữa "tìm một lần rồi thôi" và "tìm nhiều vòng có phản hồi".
- Bài của Anthropic cũng tự đi đến hướng lai: nạp sẵn một ít dữ liệu cho nhanh (ví dụ file hướng dẫn dự án luôn được đưa vào context), phần còn lại để agent tự khám phá.

Cách đọc mà mình thấy hợp lý: vòng lặp là thứ tạo khác biệt lớn, còn dùng tool tìm kiếm nào bên trong vòng lặp thì tuỳ dữ liệu và quy mô.

### Chỗ agentic search hỏng

- **Chậm và tốn token.** Mỗi vòng là một lần gọi model, và mỗi lần gọi phải gửi lại toàn bộ lịch sử. Một câu hỏi cần 8 vòng tìm sẽ chậm hơn hẳn một lần truy xuất vector vài chục mili giây.
- **Kết quả tool phình context.** Một lệnh grep vụng trả về hàng nghìn dòng. Tool phải cắt bớt kết quả, và thông báo rõ là đã cắt để model biết thu hẹp truy vấn.
- **Dừng sớm hoặc không chịu dừng.** Model có thể hài lòng với kết quả đầu tiên trông hợp lý, hoặc lòng vòng tìm thứ không tồn tại. Cần giới hạn số vòng và hướng dẫn rõ khi nào được kết luận "không tìm thấy".
- **Không có từ khóa để bám.** Câu hỏi khái niệm trên kho văn bản tự nhiên ("các trường hợp công ty từng xử lý khách phàn nàn về giao hàng trễ") khó diễn đạt thành regex. Đây là chỗ một tool tìm theo nghĩa phát huy tác dụng.
- **Quy mô.** Grep trên vài nghìn file là tức thì. Trên hàng triệu tài liệu thì cần index, dù là index từ khóa hay vector.

## Phương án thứ ba: nhét thẳng vào context

Bài Contextual Retrieval mở đầu bằng một lời khuyên dễ bị bỏ qua: nếu kho kiến thức nhỏ hơn khoảng 200.000 token (họ ước lượng cỡ 500 trang), hãy đưa toàn bộ vào prompt và khỏi cần RAG. Kết hợp prompt caching thì phần tài liệu cố định ở đầu prompt được tính giá đọc cache ở các lần gọi sau, rẻ hơn nhiều so với giá input thường (mức cụ thể tuỳ nhà cung cấp và model, xem bảng giá hiện hành).

Điều kiện để cách này hiệu quả:

- Tài liệu đặt ở đầu prompt, câu hỏi đặt cuối, và phần đầu phải giống nhau từng byte giữa các lần gọi. Chèn ngày giờ hay ID phiên vào trước tài liệu là cache hỏng.
- Lưu lượng đủ đều. Cache có thời hạn (với Claude API mặc định là 5 phút, được làm mới mỗi lần đọc trúng). Mỗi ngày hỏi vài câu rải rác thì hầu như lần nào cũng trả giá ghi cache.
- Tài liệu ít đổi. Mỗi lần sửa nội dung là ghi cache lại từ điểm sửa trở đi.

Giới hạn: context rot vẫn áp dụng, nên "vừa context window" không đồng nghĩa "model dùng tốt hết". Với câu hỏi cần đúng một chi tiết nhỏ trong vài trăm trang, mình vẫn sẽ đo so sánh với một pipeline retrieval trước khi tin.

## Chọn cái nào

| Tình huống | Hướng nên thử trước | Lý do |
| --- | --- | --- |
| Kho nhỏ (dưới vài trăm nghìn token), ít đổi, hỏi thường xuyên | Nhét cả vào context + prompt caching | Không có khâu retrieval để hỏng; chi phí được cache gánh |
| Codebase, thư mục tài liệu có cấu trúc, dữ liệu đổi liên tục | Agentic search với grep/glob/đọc file | Định danh chính xác, không lo index cũ, có vòng sửa sai |
| Kho lớn văn bản tự nhiên, cần trả lời nhanh, câu hỏi một bước | Hybrid (BM25 + embedding) + reranker | Latency thấp và ổn định, chi phí mỗi truy vấn dự đoán được |
| Như trên nhưng chunk hay thiếu ngữ cảnh (hợp đồng, báo cáo, quy chế) | Thêm contextual retrieval | Nhắm đúng lỗi chunk mất ngữ cảnh, trả phí một lần lúc lập index |
| Kho lớn, câu hỏi nhiều bước hoặc cần tổng hợp | Agent với tool tìm kiếm hybrid | Vòng lặp lo phần nhiều bước, index lo phần quy mô |
| Dữ liệu có phân quyền theo người dùng | Lọc quyền ngay trong tool hoặc truy vấn, trước khi kết quả đến model | Model không phải là lớp kiểm soát truy cập |

Thứ tự mình khuyên khi bắt đầu một dự án: thử phương án đơn giản nhất còn khả thi (nhét context, rồi agentic search trên file), dựng bộ eval, và chỉ thêm index khi số đo cho thấy cần.

## Ghi chú cho tiếng Việt

Phần này mình chỉ nêu những điều suy ra được từ cơ chế hoặc từ tài liệu của công cụ. Mình không có benchmark tiếng Việt đủ tin cậy để trích.

**Tách từ ảnh hưởng trực tiếp đến BM25.** Tiếng Việt viết cách nhau theo âm tiết, không theo từ. Một bộ phân tích mặc định tách theo khoảng trắng sẽ coi "học sinh" là hai token "học" và "sinh". Hệ quả: với BM25 kiểu túi từ, "học sinh" và "sinh học" có cùng tập token, và từ "sinh" trong "sinh viên", "vệ sinh", "sinh nhật" đều góp điểm cho truy vấn "sinh học". Có hai hướng xử lý:

- Tách từ trước khi index bằng thư viện như underthesea (hàm `word_tokenize` của nó gộp "Chàng trai", "Quảng Trị", "khởi nghiệp" thành từng từ), và áp dụng đúng cùng một bộ tách cho truy vấn. Rủi ro là bộ tách từ cũng sai, nhất là với tên riêng và thuật ngữ chuyên ngành, và sai lệch giữa lúc index với lúc truy vấn sẽ làm mất kết quả.
- Giữ token âm tiết nhưng index thêm cặp âm tiết liền nhau (bigram/shingle), để "học sinh" đúng thứ tự được điểm cao hơn. Đơn giản hơn, index to hơn.

Mình không biết hướng nào thắng trên dữ liệu của bạn. Đây đúng là thứ nên đo bằng recall@k thay vì tranh luận.

**Dấu và chuẩn hóa Unicode.** Người dùng hay gõ không dấu ("hoa don dien tu"). BM25 khớp theo chuỗi nên truy vấn không dấu sẽ không khớp tài liệu có dấu, trừ khi bạn index thêm một trường đã bỏ dấu và tìm trên cả hai. Ngoài ra, cùng một chữ có dấu có thể được mã hóa dạng dựng sẵn (NFC) hoặc dạng tổ hợp (NFD), nhìn giống hệt nhau nhưng khác byte. Chuẩn hóa về một dạng (thường là NFC) ở cả khâu nạp tài liệu và khâu truy vấn, nếu không thì khớp chuỗi và grep sẽ trượt một cách khó hiểu.

**Embedding và reranker phải là model đa ngữ.** Model huấn luyện chủ yếu trên tiếng Anh sẽ cho vector kém với tiếng Việt. BGE-M3 là một lựa chọn mã nguồn mở hay được nhắc: trang model ghi hỗ trợ hơn 100 ngôn ngữ, đầu vào tới 8.192 token, và sinh được cả vector dense lẫn sparse (chính trang đó cũng khuyến nghị hybrid cộng rerank). Các nhà cung cấp embedding thương mại cũng có model đa ngữ. Mình không khẳng định model nào tốt nhất cho tiếng Việt: hãy lấy vài chục câu hỏi thật của hệ thống bạn và đo. Nhớ kiểm tra riêng trường hợp truy vấn không dấu và truy vấn trộn tiếng Anh.

**Agentic search với grep** chịu ảnh hưởng của cả hai vấn đề trên: model có thể grep "hóa đơn" trong khi file ghi "hoá đơn" (hai cách đặt dấu đều phổ biến), hoặc file ở dạng NFD. Chuẩn hóa dữ liệu lúc nạp và ghi chú các biến thể chính tả vào hướng dẫn cho agent sẽ giúp nhiều.

## Đo retrieval: tách hai khâu ra

Lỗi phổ biến nhất khi đánh giá hệ RAG là chỉ nhìn câu trả lời cuối. Câu trả lời sai có hai nguyên nhân khác hẳn nhau: không tìm ra đoạn đúng, hoặc tìm ra rồi nhưng model dùng sai. Cách sửa của hai trường hợp không liên quan gì đến nhau, nên phải đo riêng.

### Đo retrieval

Cần một bộ câu hỏi kèm danh sách chunk (hoặc tài liệu) được coi là đúng. Vài chục đến một trăm câu lấy từ câu hỏi thật là đủ để bắt đầu. Nhớ gồm cả câu hỏi mà kho không có đáp án.

- **recall@k**: trong số chunk đúng, bao nhiêu phần trăm lọt vào top k. Đây là chỉ số quan trọng nhất cho khâu lấy ứng viên, vì thứ không lọt vào top k thì các khâu sau không cứu được. Con số "tỷ lệ hỏng" của Anthropic ở trên chính là 1 trừ recall@20.
- **precision@k**: trong top k có bao nhiêu phần trăm thực sự liên quan. Đo mức nhiễu đưa cho model.
- **MRR, nDCG**: có tính đến thứ hạng, dùng khi so sánh reranker.

```python
def recall_at_k(retrieved_ids, relevant_ids, k):
    hit = len(set(retrieved_ids[:k]) & set(relevant_ids))
    return hit / len(relevant_ids)
```

### Đo generation

Giữ cố định context (lý tưởng là context chứa sẵn đoạn đúng) rồi chấm câu trả lời theo hai trục:

- **Bám nguồn**: mọi khẳng định trong câu trả lời có nằm trong context được cấp không.
- **Đúng**: câu trả lời có khớp đáp án tham chiếu không.

Một hệ thống có thể bám nguồn tuyệt đối mà vẫn sai, khi nguồn được lấy về là sai. Vì vậy mới cần cả hai khâu đo.

### Đọc kết quả

| recall@k | Câu trả lời | Nguyên nhân có khả năng | Sửa ở đâu |
| --- | --- | --- | --- |
| Thấp | Sai | Không tìm ra | Chunking, hybrid, rewriting, embedding model |
| Cao | Sai | Tìm ra nhưng dùng sai hoặc bị nhiễu lấn | Rerank, giảm top-n, prompt, model |
| Thấp | Đúng | Model trả lời từ kiến thức huấn luyện | Nguy hiểm: sẽ sai khi dữ liệu của bạn khác kiến thức chung |
| Cao | Đúng | Ổn | |

### Với agentic search

Không có "top-k" cố định, nên đo theo quỹ đạo: agent có mở đúng file hoặc đúng đoạn chứa đáp án không, mất bao nhiêu vòng, tốn bao nhiêu token và thời gian cho mỗi câu hỏi. Hai con số sau quan trọng ngang độ chính xác, vì đó chính là cái giá của phương pháp. Kết quả giữa các lần chạy cũng dao động nhiều hơn pipeline cố định, nên mỗi câu nên chạy vài lần.

## Checklist thiết kế

- Có bộ câu hỏi kèm nhãn để đo recall@k trước khi tối ưu bất cứ thứ gì không?
- Truy vấn có chứa mã, tên riêng, số hiệu không? Có thì cần BM25 hoặc grep, embedding một mình không đủ.
- Hệ thống làm gì khi không tìm thấy gì liên quan? Có ngưỡng điểm hoặc hướng dẫn để model nói "không có trong tài liệu" không?
- Phân quyền được áp ở tầng truy vấn, trước khi dữ liệu đến model chưa?
- Với tiếng Việt: đã chuẩn hóa Unicode, xử lý truy vấn không dấu, và thống nhất bộ tách từ giữa index với truy vấn chưa?
- Với agent: kết quả tool có bị giới hạn kích thước không, có giới hạn số vòng không?

## Điều còn mở / chưa chắc

- **Index vector có còn cần cho code không.** Claude Code và Cursor đưa ra hai câu trả lời khác nhau, đều dựa trên đánh giá nội bộ. Chưa có so sánh độc lập nào mình biết mà kiểm soát được cả model, harness lẫn quy mô codebase.
- **Context dài đến đâu thì retrieval hết cần.** Context window 1 triệu token đã phổ biến, nhưng context rot là có thật và mức độ thay đổi theo từng thế hệ model. Ngưỡng 200.000 token trong bài của Anthropic là của năm 2024, mình không chắc con số hợp lý hiện nay là bao nhiêu.
- **Contextual retrieval trên tiếng Việt.** Cơ chế không phụ thuộc ngôn ngữ, nhưng mình chưa thấy số đo công khai nào cho tiếng Việt.
- **Tách từ hay bigram âm tiết cho BM25 tiếng Việt.** Mình nêu cả hai vì không có căn cứ để chọn hộ bạn.

## Nguồn đọc thêm

- [Introducing Contextual Retrieval (Anthropic)](https://www.anthropic.com/news/contextual-retrieval): nguồn của bảng số liệu ở trên, kèm prompt tạo ngữ cảnh và lời khuyên bỏ RAG khi kho nhỏ.
- [Effective context engineering for AI agents (Anthropic)](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents): lập luận cho hướng "just in time" và hướng lai, cùng khái niệm context rot.
- [Phỏng vấn nhóm Claude Code trên Latent Space](https://www.latent.space/p/claude-code): phần Boris Cherny giải thích vì sao bỏ RAG, đọc để thấy mức độ chắc chắn thật sự của nhận định đó.
- [Bài của Cursor về semantic search cho agent](https://cursor.com/blog/semsearch): quan điểm ngược lại, có số liệu A/B, nên đọc cùng nguồn trên.
- [Lost in the Middle (Liu và cộng sự)](https://arxiv.org/abs/2307.03172): thí nghiệm gốc về việc vị trí thông tin trong context ảnh hưởng độ chính xác.
- [Precise Zero-Shot Dense Retrieval without Relevance Labels (HyDE)](https://arxiv.org/abs/2212.10496): paper gốc của kỹ thuật embed câu trả lời giả định.
- [BAAI/bge-m3 trên Hugging Face](https://huggingface.co/BAAI/bge-m3): mô tả một embedding model đa ngữ có cả dense và sparse, điểm xuất phát tốt để thử với tiếng Việt.
- [underthesea](https://github.com/undertheseanlp/underthesea): thư viện xử lý tiếng Việt có tách từ, dùng khi thử hướng tách từ cho BM25.
- [Prompt caching (tài liệu Claude API)](https://platform.claude.com/docs/en/build-with-claude/prompt-caching): cơ chế và giá cache, cần đọc trước khi chọn phương án nhét cả tài liệu vào context.
- [Context windows (tài liệu Claude API)](https://platform.claude.com/docs/en/build-with-claude/context-windows): chỗ tài liệu chính thức nói về context rot.
- Cormack, Clarke, Büttcher (2009), "Reciprocal Rank Fusion outperforms Condorcet and individual rank learning methods": bài báo gốc của RRF (mình không kiểm tra được link trong lúc viết nên chỉ ghi tên).
