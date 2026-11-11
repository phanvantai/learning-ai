---
title: "LLM dưới nắp máy: những gì kỹ sư cần biết ở tầng dưới API"
date: 2026-11-11
tags: [LLM, Context engineering]
description: Token, sinh tự hồi quy, sampling, constrained decoding, thinking, prompt caching, latency và batch, nhìn từ góc người thiết kế hệ thống. Kèm phép tính chi phí cho một agent loop 30 lượt có và không có caching.
loai: chuyen-sau
---

Gọi API của một LLM trông như gọi một hàm nhận chuỗi, trả chuỗi. Nhiều quyết định thiết kế sai trong hệ thống dùng LLM bắt nguồn từ việc tin vào hình ảnh đó: coi model như có trí nhớ, coi temperature 0 là tất định, coi JSON hợp lệ là JSON đúng, coi input là rẻ. Bài này đi qua những cơ chế nằm ngay dưới API mà bạn không cần biết để gọi được, nhưng cần biết để tính đúng chi phí, độ trễ và độ tin cậy.

Ví dụ và số liệu chủ yếu lấy từ tài liệu Claude API vì tài liệu này mô tả cơ chế khá chi tiết, có đối chiếu OpenAI và Gemini ở chỗ khác biệt đáng kể. Giá và giới hạn đổi nhanh, nên mọi con số ở đây là tại thời điểm viết (tháng 11/2026), kèm link để bạn kiểm tra lại.

## Bức tranh chung

```text
  request: tools + system + messages (toàn bộ lịch sử)
                    |
                    v
            [ tokenizer ]  văn bản --> dãy số nguyên (token)
                    |
                    v
   +--------------------------------------+
   |  model: đọc cả dãy token, tính phân  |
   |  phối xác suất cho token KẾ TIẾP     |<-----+
   +--------------------------------------+      |
                    |                            |
                    v                            |
   [ sampling: temperature, top-p, grammar ]     |
                    |                            |
                    v                            |
        chọn 1 token, nối vào cuối dãy ----------+
                    |
                    v   (dừng khi gặp token kết thúc, stop sequence, hoặc chạm max_tokens)
            [ detokenizer ] --> response
```

Mọi mục bên dưới là hệ quả của sơ đồ này.

## Token: đơn vị tính tiền, và tiếng Việt tốn hơn bạn nghĩ

Model không đọc ký tự hay từ. Tokenizer cắt văn bản thành các mảnh (token) theo một bảng từ vựng học từ dữ liệu, phổ biến nhất là họ thuật toán BPE: chuỗi byte nào xuất hiện cùng nhau thường xuyên thì được gộp thành một token. Từ tiếng Anh phổ biến thường là một token. Chuỗi hiếm bị cắt nhỏ, có khi đến từng byte.

Hệ quả cho tiếng Việt: chữ có dấu chiếm 2-3 byte trong UTF-8, và tần suất xuất hiện trong dữ liệu huấn luyện tokenizer thấp hơn tiếng Anh, nên dễ bị cắt vụn hơn. Mức độ phụ thuộc hoàn toàn vào tokenizer. Đây là kết quả chạy đoạn code dưới với thư viện tiktoken (tokenizer mã nguồn mở của OpenAI) trên hai câu cùng nghĩa:

```python
import tiktoken, unicodedata

vi = ("Prompt caching cho phép nhà cung cấp lưu lại phần đầu của prompt đã xử lý, "
      "để các yêu cầu sau có cùng phần đầu được tính giá rẻ hơn và phản hồi nhanh hơn. "
      "Điều kiện là phần đầu phải trùng khớp tuyệt đối với lần trước.")
en = ("Prompt caching lets the provider store the already processed beginning of a prompt, "
      "so that later requests sharing the same beginning are billed at a lower price and "
      "answered faster. The condition is that the beginning must match the previous one exactly.")

for name in ["cl100k_base", "o200k_base"]:
    enc = tiktoken.get_encoding(name)
    print(name, len(enc.encode(vi)), len(enc.encode(en)),
          len(enc.encode(unicodedata.normalize("NFD", vi))))
```

| Tokenizer | Tiếng Việt (NFC), 48 âm tiết | Tiếng Anh, 42 từ | Tiếng Việt dạng NFD |
| --- | --- | --- | --- |
| `cl100k_base` (thế hệ cũ) | 98 token | 45 token | 206 token |
| `o200k_base` (thế hệ mới hơn) | 55 token | 45 token | 161 token |

Ba điều rút ra, kèm giới hạn của phép đo (một mẫu, hai tokenizer của một hãng):

1. **Khoảng cách Việt-Anh phụ thuộc thế hệ tokenizer.** Trên bảng từ vựng cũ, cùng nội dung tiếng Việt tốn hơn gấp đôi. Trên bảng mới hơn, chênh lệch còn khoảng 20%.
2. **Chuẩn hóa Unicode ảnh hưởng trực tiếp đến hóa đơn.** Cùng một chữ có dấu có thể được mã hóa dạng dựng sẵn (NFC) hoặc dạng ký tự gốc cộng dấu rời (NFD). Mắt người không phân biệt được, nhưng trong phép đo trên, dạng NFD tốn gấp 2-3 lần token. Văn bản NFD hay đến từ tên file trên một số hệ điều hành và từ vài công cụ trích xuất PDF. Chuẩn hóa về NFC trước khi gửi là một dòng code.
3. **Mỗi model một tokenizer, đừng đếm chéo.** tiktoken không đếm đúng cho Claude hay Gemini. Tài liệu Claude ghi rằng các model từ Opus 4.7 trở đi dùng tokenizer mới, sinh nhiều hơn khoảng 30% token cho cùng văn bản so với model trước đó. Tức là nâng cấp model có thể làm đổi số token của đúng prompt cũ. Claude API có endpoint đếm token miễn phí (`count_tokens`), nhận đúng cấu trúc request thật gồm cả system prompt và tool. Tài liệu ghi rõ con số trả về là ước lượng, có thể lệch nhẹ so với thực tế.

## Sinh tự hồi quy và ba hệ quả

Model sinh từng token một. Mỗi token mới cần một lượt tính toán trên toàn bộ dãy phía trước, và token vừa sinh được nối vào dãy để sinh token kế tiếp. Từ đây suy ra ba điều.

**Output đắt và chậm hơn input.** Input được xử lý song song trong một lượt. Output phải sinh tuần tự. Bảng giá phản ánh điều đó: với Claude Sonnet 5.5, input giá 2 USD và output giá 10 USD cho mỗi triệu token, tức gấp 5 lần. Thời gian phản hồi cũng tỷ lệ gần tuyến tính với số token output.

**Model không có trạng thái.** Giữa hai lần gọi, model không nhớ gì. "Hội thoại" là ảo giác do phía client tạo ra bằng cách gửi lại toàn bộ lịch sử ở mỗi lượt.

**Trong agent loop, chi phí input tăng theo bình phương số lượt.** Agent chạy vòng lặp: model gọi tool, nhận kết quả, gọi tiếp. Mỗi lượt là một request gửi lại mọi thứ từ đầu. Nếu phần cố định (system prompt và định nghĩa tool) dài `P` token và mỗi lượt thêm `d` token (output của model cộng kết quả tool), thì lượt thứ `i` gửi `P + (i-1)·d` token input. Tổng sau `N` lượt:

```text
tổng input = N·P + d·N·(N-1)/2
```

Số hạng thứ hai tăng theo `N²`. Đây là lý do một agent chạy 100 lượt không đắt gấp 10 lần agent chạy 10 lượt mà đắt hơn nhiều.

## Phép tính: agent loop 30 lượt, có và không có caching

Giả định minh họa (số token là mình tự đặt cho dễ tính, giá lấy từ bảng giá Claude Sonnet 5.5 tại thời điểm viết):

- `P` = 10.000 token cố định (system prompt, định nghĩa tool, yêu cầu ban đầu).
- Mỗi lượt model sinh 500 token output, tool trả về 1.500 token. Vậy `d` = 2.000.
- `N` = 30 lượt, các lượt cách nhau dưới 5 phút.
- Giá mỗi triệu token: input 2 USD, ghi cache (thời hạn 5 phút) 2,50 USD, đọc cache 0,10 USD, output 10 USD.

```python
P, d, N, out = 10_000, 2_000, 30, 500
IN, WRITE, READ, OUT = 2.00, 2.50, 0.10, 10.00   # USD / 1 triệu token

# Không cache: lượt i gửi P + (i-1)*d token, tính giá input thường
no_cache = sum(P + (i - 1) * d for i in range(1, N + 1))

# Có cache: lượt 1 ghi P. Từ lượt 2, đọc lại phần đã có và ghi thêm d token mới.
write = P + (N - 1) * d
read = sum(P + (i - 2) * d for i in range(2, N + 1))

print(no_cache * IN / 1e6)                       # 2.34
print(write * WRITE / 1e6 + read * READ / 1e6)   # 0.2802
print(N * out * OUT / 1e6)                       # 0.15
```

| | Không cache | Có cache |
| --- | --- | --- |
| Token input phải xử lý | 1.170.000 | 1.170.000 (68.000 ghi + 1.102.000 đọc) |
| Tiền input | 2,34 USD | 0,28 USD |
| Tiền output (15.000 token) | 0,15 USD | 0,15 USD |
| Tổng | 2,49 USD | 0,43 USD |

Những điều phép tính này cho thấy:

- Không cache, 94% chi phí là input, dù output đắt gấp 5 lần tính theo token. Trong agent loop, thứ bạn trả tiền chủ yếu là việc đọc lại lịch sử.
- Caching không giảm số token, nó đổi giá của chúng. Số token vẫn chiếm chỗ trong context window như cũ.
- Một lần trượt cache giữa chừng là đắt. Giả sử trước lượt 20 có khoảng dừng quá 5 phút: thay vì đọc 46.000 token với giá khoảng 0,005 USD, bạn ghi lại toàn bộ với giá khoảng 0,12 USD. Một lần thì không sao. Nhưng nếu harness của bạn vô tình làm hỏng cache ở mọi lượt (xem mục dưới), bạn quay về cột bên trái cộng thêm 25% phí ghi.
- Mô hình này đã đơn giản hóa: bỏ qua thinking token, coi mọi lượt dài bằng nhau, và coi mọi lượt đều trúng cache.

## Prompt caching hoạt động thế nào

Về nguyên lý, cache khả thi vì cách model đọc: trạng thái tính toán tại mỗi vị trí chỉ phụ thuộc vào các token đứng trước nó. Hai request có chung phần đầu thì phần tính toán cho đoạn đó giống hệt nhau và có thể dùng lại. Nhà cung cấp không công bố chi tiết triển khai, nhưng nguyên lý này giải thích quy tắc quan trọng nhất: **cache khớp theo tiền tố**. Đổi một byte ở vị trí nào thì mọi thứ từ vị trí đó trở về sau phải tính lại.

Chi tiết với Claude API, theo tài liệu hiện hành:

- **Thứ tự ghép prompt** là `tools`, rồi `system`, rồi `messages`. Đổi định nghĩa một tool là mất cache của tất cả.
- **Thời hạn (TTL)** mặc định 5 phút, được làm mới miễn phí mỗi lần đọc trúng. Có lựa chọn 1 giờ với giá ghi cao hơn.
- **Giá**: ghi cache 5 phút bằng 1,25 lần giá input, ghi cache 1 giờ bằng 2 lần, đọc cache bằng 0,1 lần (một số model mới còn thấp hơn, xem bảng giá).
- **Độ dài tối thiểu** để được cache tuỳ model, từ 512 đến 4.096 token. Prompt ngắn hơn ngưỡng vẫn chạy bình thường, không báo lỗi, chỉ là không cache.
- **Điểm đánh dấu**: bạn đặt `cache_control` lên block cuối cùng của phần muốn cache (tối đa 4 điểm), hoặc bật chế độ tự động để điểm đánh dấu tự dời theo cuối hội thoại.
- **Kiểm tra**: mỗi response trả về `cache_creation_input_tokens` (ghi), `cache_read_input_tokens` (đọc) và `input_tokens` (phần không cache). Nếu số đọc bằng 0 qua nhiều request liên tiếp có cùng phần đầu, có thứ gì đó đang phá tiền tố.

Các nhà cung cấp khác cùng nguyên lý tiền tố nhưng khác cách dùng. OpenAI bật cache mặc định, không cần đánh dấu, tài liệu ghi giá đọc cache bằng 0,1 lần giá input (thấp hơn ở vài model mới) và thời hạn tuỳ thế hệ model. Gemini có cache ngầm bật mặc định trên các model từ 2.5 trở đi. Chi tiết xem link cuối bài.

### Những thứ âm thầm phá cache

- Ngày giờ hiện tại, ID phiên, tên người dùng nằm trong system prompt. Chuyển chúng xuống cuối, vào một tin nhắn sau điểm đánh dấu.
- Danh sách tool sinh từ một cấu trúc không có thứ tự ổn định, hoặc bật tắt tool theo từng lượt.
- JSON được serialize với thứ tự khóa thay đổi giữa các lần.
- Sửa hoặc xóa tin nhắn cũ trong lịch sử (ví dụ tự cắt bớt kết quả tool cũ để tiết kiệm). Tiết kiệm được vài nghìn token nhưng mất cache của toàn bộ phần sau điểm sửa.
- Đổi tham số có ảnh hưởng đến prompt được dựng phía server. Tài liệu Claude liệt kê `tool_choice`, cấu hình thinking và mức effort trong số đó.

Nguyên tắc thiết kế rút ra: coi lịch sử hội thoại là cấu trúc chỉ nối thêm (append-only), xếp nội dung theo thứ tự từ ổn định nhất đến hay đổi nhất, và theo dõi tỷ lệ token đọc cache như một chỉ số vận hành.

## Sampling, và vì sao temperature 0 vẫn không tất định

Ở mỗi bước, model cho ra một điểm số (logit) cho từng token trong bảng từ vựng. Sampling là cách chọn một token từ đó:

- **Temperature** chia logit trước khi chuyển thành xác suất. Giá trị thấp làm phân phối nhọn hơn (token có xác suất cao nhất càng áp đảo), giá trị cao làm phẳng ra. Temperature 0 về lý thuyết là luôn chọn token cao nhất.
- **Top-p** (nucleus sampling) chỉ giữ nhóm token nhỏ nhất có tổng xác suất đạt `p`, bỏ phần đuôi, rồi chọn trong nhóm đó.

Điều nhiều người ngạc nhiên: temperature 0 không bảo đảm cùng input cho cùng output. Tài liệu Claude viết thẳng rằng ngay cả với temperature 0, kết quả không hoàn toàn tất định và cùng một input có thể cho output khác nhau giữa các lần gọi, kể cả khi gọi qua các nền tảng cloud bên thứ ba. Giải thích thường gặp trong cộng đồng (mình không có nguồn chính thức nêu chi tiết): phép tính số thực dấu phẩy động cho kết quả lệch rất nhỏ tuỳ phần cứng và cách các request được gộp lô, và khi hai token đứng đầu có điểm gần bằng nhau thì sai lệch nhỏ đó đủ để đổi lựa chọn. Vì sinh tự hồi quy, một token khác đi ở vị trí 50 kéo theo toàn bộ phần sau khác đi.

Hệ quả thiết kế:

- Đừng viết test so khớp nguyên văn output của model. So khớp theo cấu trúc, theo tiêu chí, hoặc dùng một bước chấm riêng.
- Cần kết quả lặp lại được (để kiểm toán, để tránh tính tiền hai lần khi retry) thì lưu output phía bạn, kèm khóa là hash của request. Đừng trông vào việc gọi lại sẽ ra kết quả cũ.
- Khi đánh giá một thay đổi prompt, chạy mỗi mẫu nhiều lần. Khác biệt giữa hai lần chạy cùng prompt có thể lớn hơn khác biệt giữa hai prompt.

## Structured output và constrained decoding

Yêu cầu "trả về JSON" trong prompt chỉ làm tăng xác suất model sinh JSON hợp lệ. Constrained decoding thì bảo đảm bằng cơ chế: schema được biên dịch thành một văn phạm, và ở mỗi bước sinh, các token làm chuỗi vi phạm văn phạm bị loại khỏi danh sách ứng viên trước khi chọn. Model không thể sinh ra dấu ngoặc thừa vì token đó không còn trong lựa chọn.

Với Claude API, tính năng này gọi là structured outputs (cho câu trả lời) và strict tool use (cho tham số tool). OpenAI có tính năng tương đương cùng tên.

Những giới hạn ghi trong tài liệu Claude mà mình thấy đáng nhớ:

- **Bảo đảm cú pháp, không bảo đảm ngữ nghĩa.** Trường `so_tien` chắc chắn là số. Nó có đúng là số tiền trong hóa đơn hay không thì vẫn là chuyện của model. Vẫn phải kiểm tra giá trị.
- **Hai trường hợp bảo đảm bị phá**: model từ chối (`stop_reason: "refusal"`) và output bị cắt do chạm `max_tokens`. Cả hai đều trả HTTP 200. Luôn kiểm tra `stop_reason` trước khi parse.
- **Lần đầu dùng một schema sẽ chậm hơn** vì phải biên dịch văn phạm. Kết quả biên dịch được giữ 24 giờ kể từ lần dùng cuối.
- **Không phải mọi tính năng JSON Schema đều được hỗ trợ**: schema đệ quy, ràng buộc số (`minimum`, `maximum`), ràng buộc độ dài chuỗi nằm trong danh sách không hỗ trợ. Schema quá phức tạp bị từ chối với lỗi 400.
- **Đổi schema làm mất prompt cache** của hội thoại đó.

Một hệ quả của sinh tự hồi quy mà schema nên tận dụng: model sinh các trường theo thứ tự, và trường sinh sau "nhìn thấy" trường sinh trước. Nếu bạn muốn model lập luận rồi mới kết luận, đặt trường lập luận trước trường kết luận. Đặt ngược lại thì kết luận đã chốt trước khi có lập luận. Lưu ý thêm là tài liệu Claude ghi rằng các trường bắt buộc được sinh trước, trường tuỳ chọn sau, nên thứ tự khai báo không phải lúc nào cũng là thứ tự sinh.

## Thinking: mua chất lượng bằng output token

Model sinh một lượt từ trái sang phải, không quay lại sửa. Thinking (tên gọi tuỳ hãng: extended thinking, reasoning) cho model một khoảng để viết nháp trước khi viết câu trả lời chính thức. Về cơ chế, đó vẫn là sinh token như thường, chỉ khác là phần nháp được tách riêng khỏi câu trả lời.

Các điểm ảnh hưởng đến thiết kế, theo tài liệu Claude:

- **Thinking được tính tiền như output**, loại token đắt nhất, kể cả khi API không trả phần nội dung thinking về cho bạn. Tài liệu ghi rõ: thứ bạn thấy (nếu có) chỉ là bản tóm tắt, còn tiền tính theo toàn bộ số token thinking đã sinh. Số output token trên hóa đơn vì thế không khớp với độ dài response bạn nhận.
- **Thinking tính vào `max_tokens`.** Đặt `max_tokens` thấp theo thói quen cũ có thể khiến model dùng gần hết cho phần nháp và câu trả lời bị cắt.
- **Thinking làm tăng thời gian đến token đầu tiên** của câu trả lời, vì phần nháp sinh trước.
- **Ở các lượt sau, thinking cũ có thể trở thành input.** Trên các model giữ lại thinking của lượt trước, những block đó nằm trong context và được tính tiền input như phần còn lại của lịch sử. Phép tính chi phí agent loop ở trên sẽ có `d` lớn hơn.
- **Trong vòng lặp tool, phải gửi trả block thinking nguyên vẹn.** Sửa hoặc bỏ chúng sẽ gây lỗi hoặc làm giảm chất lượng.
- **Cách điều khiển đã đổi.** Tham số ngân sách token cố định (`budget_tokens`) được tài liệu đánh dấu là tính năng cũ. Các model mới dùng chế độ thích ứng: model tự quyết định có nghĩ không và nghĩ bao nhiêu, bạn điều chỉnh qua mức effort. Đổi mức effort giữa hội thoại làm mất prompt cache.

Đánh đổi nói gọn: thinking mua độ chính xác trên bài khó bằng tiền và độ trễ. Với việc phân loại đơn giản hay trích xuất theo mẫu, nó thường là chi phí thừa. Cách duy nhất để biết là đo trên việc của bạn ở vài mức effort.

## Vì sao hallucination là hệ quả cấu trúc

Nhìn lại sơ đồ: ở mỗi bước model phải đưa ra một phân phối trên các token kế tiếp và một token sẽ được chọn. Không có nhánh nào trong cơ chế này là "tra cứu, không thấy thì dừng". Một câu sai sự thật và một câu đúng sự thật được sinh ra bằng đúng một quy trình, và với model cả hai đều là chuỗi token có xác suất cao. "Tôi không biết" cũng chỉ là một chuỗi token, được sinh ra khi nó có xác suất cao hơn các phương án khác.

Paper "Why Language Models Hallucinate" (Kalai và cộng sự, 2025) lập luận thêm một tầng: cách huấn luyện và cách chấm điểm phổ biến thưởng cho việc đoán hơn là thừa nhận không chắc, giống học sinh làm bài thi mà bỏ trống thì chắc chắn không có điểm. Theo các tác giả, phần lớn benchmark phạt câu trả lời thể hiện sự không chắc chắn, nên model được tối ưu để đoán cho trôi chảy. Họ đề xuất sửa cách chấm của các benchmark hiện có thay vì thêm benchmark riêng về hallucination.

Với người thiết kế hệ thống, điều này có nghĩa là hallucination không thể tắt bằng một câu trong prompt. Nó được quản lý ở tầng hệ thống:

- Đưa nguồn vào context và yêu cầu trả lời dựa trên nguồn, có trích dẫn đoạn cụ thể để kiểm tra được.
- Cho model lối ra hợp lệ: nói rõ rằng "không có trong tài liệu" là câu trả lời được chấp nhận.
- Kiểm tra bằng code những gì kiểm tra được (ID có tồn tại không, số có khớp tổng không, trích dẫn có nằm nguyên văn trong nguồn không).
- Với hành động có hậu quả, đặt một bước xác nhận không phụ thuộc vào chính model đó.

## Giới hạn output token

Context window lớn không có nghĩa output lớn. Tài liệu Claude ghi các model hiện tại có context window 1 triệu token nhưng một request chỉ sinh được tối đa 128.000 token output. `max_tokens` là giới hạn cứng do bạn đặt, và model không biết đến nó: chạm giới hạn là bị cắt ngang, có thể giữa câu hoặc giữa một chuỗi JSON.

Việc cần làm:

- Luôn đọc `stop_reason`. Giá trị `max_tokens` nghĩa là output chưa xong, đừng coi nó là kết quả hoàn chỉnh.
- Output dài (báo cáo, file code lớn) nên sinh theo từng phần có ranh giới rõ, hoặc cho model ghi ra file qua tool, thay vì trông vào một response khổng lồ.
- Request có output dài cần streaming. Giữ một kết nối HTTP im lặng nhiều phút rất dễ gặp timeout ở đâu đó giữa đường.

## Latency: TTFT và tốc độ sinh là hai con số khác nhau

Thời gian chờ của một request gồm hai phần:

```text
tổng thời gian ≈ TTFT + (số token output / tốc độ sinh)
```

- **TTFT** (time to first token): từ lúc gửi đến lúc nhận token đầu tiên. Gồm thời gian mạng, xếp hàng, và xử lý toàn bộ input. Prompt dài hơn thì TTFT dài hơn. Thinking cũng nằm ở đây nếu bạn tính theo token đầu tiên của câu trả lời.
- **Tốc độ sinh** (token mỗi giây): phụ thuộc model và tải hệ thống. Bạn không điều khiển trực tiếp, trừ việc chọn model hoặc trả thêm cho các chế độ nhanh nếu nhà cung cấp có.

Ví dụ với số giả định: TTFT 1 giây, tốc độ 60 token/giây. Câu trả lời 150 token mất khoảng 3,5 giây. Câu trả lời 3.000 token mất khoảng 51 giây. Với hầu hết ứng dụng, số token output là thứ quyết định độ trễ, và là thứ rẻ nhất để cắt.

Các đòn bẩy, xếp theo mức dễ làm:

1. **Giảm output.** Yêu cầu trả lời ngắn, bỏ phần diễn giải không ai đọc, trả về ID thay vì chép lại nội dung.
2. **Streaming.** Không làm request nhanh hơn, nhưng người dùng thấy chữ sau thời gian TTFT thay vì sau tổng thời gian.
3. **Prompt caching.** Phần tiền tố đọc từ cache không phải xử lý lại, nên TTFT giảm với prompt dài. Tài liệu Claude mô tả caching là cách giảm cả chi phí lẫn độ trễ.
4. **Chọn model nhỏ hơn hoặc mức effort thấp hơn** cho việc không cần model lớn.
5. **Trong agent loop**, độ trễ là tổng của mọi lượt cộng thời gian chạy tool. Cho model gọi nhiều tool song song trong một lượt, và thiết kế tool để một lần gọi làm được nhiều việc, giảm số lượt là giảm cả tiền lẫn thời gian.

Khi đo, ghi riêng TTFT và tốc độ sinh, theo phân vị (p50, p95) chứ đừng lấy trung bình. Hai con số này hỏng vì những lý do khác nhau.

## Batch API: giảm một nửa giá khi không cần trả lời ngay

Nếu việc của bạn không có người ngồi chờ (chấm eval, phân loại tồn đọng, sinh mô tả cho cả danh mục, tạo ngữ cảnh cho chunk khi lập index), batch API đổi độ trễ lấy giá. Với Claude, theo tài liệu:

- Giảm 50% cả input lẫn output.
- Phần lớn batch xong trong vòng 1 giờ, nhưng cam kết là 24 giờ, quá hạn thì các request chưa xử lý bị hết hạn.
- Một batch tối đa 100.000 request hoặc 256 MB.
- Kết quả giữ 29 ngày, và trả về không theo thứ tự gửi. Phải ghép bằng `custom_id`.
- Kết hợp được với prompt caching (hai mức giảm cộng dồn), nhưng vì các request chạy đồng thời nên trúng cache chỉ ở mức "cố gắng tối đa". Tài liệu gợi ý dùng cache 1 giờ cho batch có chung phần đầu.

Điểm thiết kế: batch cần mã xử lý kết quả từng phần. Một batch có thể có request thành công, request lỗi và request hết hạn lẫn với nhau, nên phải có đường retry cho từng `custom_id`.

## Checklist thiết kế

- Đã đếm token bằng công cụ của đúng model, trên dữ liệu thật (gồm cả tiếng Việt đã chuẩn hóa NFC) chưa?
- Đã ước lượng chi phí agent loop theo công thức bậc hai, với số lượt ở phân vị cao chứ không phải trung bình chưa?
- Phần đầu prompt có ổn định từng byte không? Có theo dõi tỷ lệ token đọc cache không?
- Mã gọi API có kiểm tra `stop_reason` trước khi dùng output không?
- Test và eval có chịu được việc output khác nhau giữa các lần chạy không?
- Giá trị trong structured output có được kiểm tra nghiệp vụ sau khi parse không?
- Việc nào không cần trả lời ngay đã được chuyển sang batch chưa?

## Điều còn mở / chưa chắc

- **Nguyên nhân chính xác của việc không tất định ở temperature 0.** Nhà cung cấp xác nhận hiện tượng nhưng không công bố chi tiết. Phần giải thích về số thực và gộp lô trong bài là cách hiểu phổ biến, mình không kiểm chứng được cho từng hãng.
- **Tiếng Việt tốn bao nhiêu token trên từng model cụ thể.** Phép đo trong bài chỉ trên hai tokenizer mã nguồn mở của OpenAI với một đoạn văn. Tokenizer của Claude và Gemini không đo được bằng cách đó, và mình không có số công khai đáng tin để trích.
- **Thinking có đáng tiền không** phụ thuộc bài toán nhiều đến mức mình không dám đưa quy tắc chung. Các hãng cũng đang đổi cách điều khiển (từ ngân sách token sang mức effort), nên kinh nghiệm cũ nhanh lỗi thời.
- **Hallucination giảm được đến đâu bằng cách đổi cách huấn luyện và chấm điểm.** Paper của Kalai và cộng sự là một lập luận lý thuyết kèm đề xuất. Việc các benchmark lớn có đổi cách chấm hay không, và model có vì thế mà biết nói "không biết" đúng lúc hơn không, vẫn chưa ngã ngũ.

## Nguồn đọc thêm

- [Bảng giá Claude API](https://platform.claude.com/docs/en/about-claude/pricing): nguồn của các mức giá trong phép tính, gồm cả giá ghi và đọc cache theo từng model. Kiểm tra lại trước khi lập ngân sách.
- [Prompt caching (Claude API)](https://platform.claude.com/docs/en/build-with-claude/prompt-caching): quy tắc tiền tố, TTL, độ dài tối thiểu và bảng những thay đổi làm mất cache.
- [Prompt caching (OpenAI)](https://developers.openai.com/api/docs/guides/prompt-caching): cùng nguyên lý nhưng tự động, đọc để thấy khác biệt về thời hạn và cách tính.
- [Context caching (Gemini API)](https://aistudio.google.com/docs/caching): mô tả cache ngầm và ngưỡng token tối thiểu của Gemini.
- [Token counting (Claude API)](https://platform.claude.com/docs/en/build-with-claude/token-counting): cách đếm token đúng cho Claude và ghi chú về tokenizer mới.
- [tiktoken](https://github.com/openai/tiktoken): thư viện dùng trong phép đo tiếng Việt ở trên, chạy lại được trên máy bạn.
- [Glossary (Claude API)](https://platform.claude.com/docs/en/about-claude/glossary): chỗ tài liệu chính thức nói temperature 0 không hoàn toàn tất định, cùng định nghĩa token và TTFT.
- [Structured outputs (Claude API)](https://platform.claude.com/docs/en/build-with-claude/structured-outputs): danh sách giới hạn schema và các trường hợp bảo đảm bị phá.
- [Thinking (Claude API)](https://platform.claude.com/docs/en/build-with-claude/thinking) và [Extended thinking](https://platform.claude.com/docs/en/build-with-claude/extended-thinking): cách tính tiền thinking, quy tắc gửi trả block, và việc chuyển từ ngân sách token sang chế độ thích ứng.
- [Context windows (Claude API)](https://platform.claude.com/docs/en/build-with-claude/context-windows): kích thước context, giới hạn output và hành vi khi tràn.
- [Reducing latency (Claude API)](https://platform.claude.com/docs/en/test-and-evaluate/strengthen-guardrails/reduce-latency): định nghĩa TTFT và các cách giảm độ trễ theo khuyến nghị chính thức.
- [Batch processing (Claude API)](https://platform.claude.com/docs/en/build-with-claude/batch-processing): giới hạn, thời hạn và cách kết hợp với caching.
- [Why Language Models Hallucinate (Kalai và cộng sự)](https://arxiv.org/abs/2509.04664): lập luận rằng hallucination đến từ cách huấn luyện và chấm điểm thưởng cho việc đoán.
- Bài "Introducing Structured Outputs in the API" của OpenAI: giải thích dễ hiểu về constrained decoding bằng văn phạm (mình không mở được link trong lúc viết nên chỉ ghi tên).
