---
title: "AI cho người làm marketing nội dung: 6 việc nên giao và cách tránh ra văn AI"
date: 2026-11-08
tags: [Marketing, Quy trình làm việc, Prompt mẫu]
description: Sáu việc lặp lại của người làm marketing nội dung có thể giao một phần cho AI, kèm câu lệnh mẫu, cách kiểm tra, và cách sửa khi bài viết ra đúng giọng "văn AI".
loai: ung-dung
---

Gõ "viết bài Facebook giới thiệu sản phẩm X" vào bất kỳ công cụ AI nào, bạn sẽ nhận về một bài đọc được, đúng chính tả, và giống hệt bài của hàng nghìn trang khác. Lý do nằm ở cách công cụ hoạt động: khi không có chất liệu riêng, nó viết ra phiên bản trung bình của mọi bài quảng cáo nó từng đọc. Người đọc lướt qua vì họ đã thấy bài đó nhiều lần rồi.

Bài này đi theo một quy trình ba bước cho mọi việc bên dưới:

```text
Người: đưa góc nhìn + chất liệu thật (bài cũ, bình luận khách, số liệu, thông số sản phẩm)
   |
AI:    dàn ý, viết nháp, tạo biến thể, đổi định dạng
   |
Người: biên tập, kiểm tra sự thật, quyết định đăng
```

Bước đầu và bước cuối là phần bạn không giao được. Mình là kỹ sư phần mềm chứ không làm marketing chuyên nghiệp, nên bài này tập trung vào chỗ mình chắc: cách đưa việc cho AI và cách kiểm tra kết quả. Chiến lược thương hiệu là việc của bạn.

## Việc nào giao, việc nào giữ

| Giao cho AI (có kiểm tra) | Giữ lại cho người |
| --- | --- |
| Rút đặc điểm giọng văn từ bài cũ | Chọn bài nào đại diện cho thương hiệu |
| Gom nhóm bình luận, đánh giá của khách | Quyết định nhắm vào nhóm khách nào |
| Xếp khung lịch nội dung | Chọn chủ đề, thông điệp, thời điểm chạy chương trình |
| Cắt một bài gốc thành nhiều định dạng | Viết (hoặc ít nhất đưa ý) cho bài gốc |
| Viết biến thể tiêu đề, câu mở, lời kêu gọi | Mọi tuyên bố về công dụng, giá, khuyến mãi |
| Diễn giải bảng số liệu đã tính xong | Tính toán số liệu và kết luận nên làm gì tiếp |

## 1. Dựng hồ sơ giọng thương hiệu từ bài cũ

Dặn AI "viết giọng trẻ trung, gần gũi" gần như không có tác dụng, vì thương hiệu nào cũng tự mô tả như vậy. Thứ có tác dụng là mẫu thật. Chọn 5 đến 10 bài cũ mà bạn thấy đúng giọng nhất (nên ưu tiên bài có tương tác tốt và do người viết), rồi nhờ AI mô tả lại giọng đó thành quy tắc cụ thể.

```text
Dưới đây là 8 bài đã đăng của thương hiệu tôi, mỗi bài cách nhau bằng dòng "---".
Hãy phân tích và viết một hồ sơ giọng văn gồm:
1. Cách xưng hô với khách (xưng gì, gọi khách là gì).
2. Độ dài câu và độ dài bài thường gặp.
3. Kiểu mở bài và kiểu kết bài hay dùng, kèm ví dụ trích nguyên văn.
4. Từ ngữ đặc trưng hay lặp lại, và những kiểu từ ngữ các bài này KHÔNG bao giờ dùng.
5. Cách nói về giá và khuyến mãi.
Mỗi nhận xét phải kèm một câu trích từ bài mẫu làm bằng chứng. Không suy diễn ngoài mẫu.

[Dán các bài vào đây]
```

Cách kiểm tra: đọc từng nhận xét và câu trích đi kèm. Nhận xét nào không có câu trích thật thì xóa. Sau đó lưu hồ sơ này cùng 2 đến 3 bài mẫu vào chỗ lưu hướng dẫn cố định của công cụ (hầu hết công cụ đều có, tên gọi mỗi nơi một khác) để không phải dán lại mỗi lần.

## 2. Nghiên cứu chân dung khách hàng từ bình luận và đánh giá thật

Nhờ AI "vẽ chân dung khách hàng mục tiêu cho tiệm bánh" thì nó sẽ bịa ra một "chị Lan, 32 tuổi, nhân viên văn phòng" nghe rất hợp lý và không dựa trên dữ liệu nào. Chân dung dùng được phải đi từ lời khách thật: đánh giá trên sàn thương mại điện tử, Google Maps, bình luận dưới bài đăng, tin nhắn hỏi hàng.

Trước khi dán, xóa tên, số điện thoại, địa chỉ của khách. Việc phân tích không cần những thứ đó.

```text
Dưới đây là khoảng 150 đánh giá và bình luận của khách về sản phẩm của tôi (đã xóa tên).
Hãy:
1. Gom thành các nhóm lý do mua, mỗi nhóm kèm số lượng bình luận và 2 câu trích nguyên văn.
2. Gom các nhóm lý do chê hoặc ngần ngại, cũng kèm số lượng và câu trích.
3. Liệt kê những từ, cụm từ khách tự dùng để gọi sản phẩm và mô tả vấn đề của họ.
4. Ghi riêng một mục "ý chỉ xuất hiện 1-2 lần" để tôi tự đánh giá.
Chỉ dựa trên dữ liệu được dán. Không thêm đặc điểm nhân khẩu học nếu bình luận không nói.
```

Mục 3 thường có giá trị nhất: cách khách tự gọi vấn đề của họ là nguyên liệu tốt cho tiêu đề và câu mở. Cách kiểm tra: tìm (Ctrl+F) vài câu trích trong file gốc để chắc AI không sửa lời khách. Số lượng mỗi nhóm coi là ước lượng, vì AI đếm trong văn bản dài hay lệch.

Nhớ giới hạn của nguồn này: người để lại đánh giá không đại diện cho toàn bộ người mua, và càng không đại diện cho người chưa mua.

## 3. Lên lịch nội dung

AI xếp khung lịch nhanh, nhưng nếu để nó tự nghĩ chủ đề thì bạn nhận về danh sách "mẹo hay", "hậu trường", "cảm nhận khách hàng" giống mọi trang khác. Hãy đưa trước các ràng buộc thật: sản phẩm nào cần đẩy, ngày nào có chương trình, câu hỏi nào khách hỏi nhiều (lấy từ việc 2), mỗi tuần làm nổi mấy bài.

```text
Lập lịch nội dung 4 tuần cho trang Facebook của [mô tả ngắn về thương hiệu].
Ràng buộc:
- 3 bài mỗi tuần, đăng thứ Ba, thứ Năm, thứ Bảy.
- Tuần 2 có chương trình [tên, thời gian]. Tuần 4 ra mắt [sản phẩm].
- Các câu hỏi khách hay hỏi: [dán danh sách].
- Nguồn lực: 1 người viết, không quay video dài.
Trả về bảng: ngày, chủ đề, góc tiếp cận, định dạng, chất liệu tôi cần chuẩn bị.
Với mỗi bài, ghi rõ nó trả lời câu hỏi nào của khách hoặc phục vụ mốc nào ở trên.
Bài nào không gắn được với câu hỏi hay mốc nào thì đừng đưa vào.
```

Cột "chất liệu tôi cần chuẩn bị" giúp bạn thấy ngay bài nào thiếu ảnh, thiếu số liệu, thiếu câu chuyện thật. Bài nào bạn không có chất liệu thì bỏ, đừng để AI viết bù bằng chữ.

## 4. Một bài gốc, nhiều định dạng

Đây là việc AI làm tốt nhất trong danh sách, vì mọi ý đã có sẵn trong bài gốc, nó chỉ đổi hình thức. Bài gốc nên do người viết hoặc ít nhất người đưa đủ ý: một bài dài trên website, một bản ghi buổi chia sẻ, một bản trả lời khách thật kỹ.

```text
Dưới đây là bài gốc và hồ sơ giọng văn của thương hiệu.
Từ bài gốc, hãy viết 3 phiên bản:
1. Bài Facebook: 120-180 chữ, hai dòng đầu phải đủ để người đọc bấm "xem thêm".
2. Email gửi khách cũ: tiêu đề dưới 50 ký tự, thân bài dưới 150 chữ, một lời kêu gọi duy nhất.
3. Kịch bản video ngắn 30-45 giây: bảng 2 cột (lời thoại / hình ảnh trên màn hình),
   3 giây đầu nêu thẳng vấn đề của người xem.
Quy tắc: chỉ dùng thông tin có trong bài gốc. Không thêm số liệu, công dụng, cam kết mới.
Nếu một phiên bản cần thông tin bài gốc không có, ghi "[CẦN BỔ SUNG: ...]" thay vì tự điền.

[Hồ sơ giọng văn]
[Bài gốc]
```

Cách kiểm tra: đặt bản phái sinh cạnh bài gốc và dò từng con số, tên sản phẩm, điều kiện khuyến mãi. Lỗi hay gặp là AI làm "gọn" một điều kiện: "giảm 20% cho đơn từ 500.000 đồng" bị rút thành "giảm 20%".

## 5. Viết biến thể quảng cáo để chạy A/B

A/B là chạy song song hai phiên bản chỉ khác nhau một yếu tố để xem bản nào hiệu quả hơn. AI giúp được ở khâu tạo nhiều phiên bản nhanh. Nó không dự đoán được bản nào thắng, và câu trả lời tự tin của nó về chuyện này không có cơ sở. Chỉ dữ liệu chạy thật mới trả lời được.

```text
Đây là mẫu quảng cáo đang chạy: [dán tiêu đề, nội dung, lời kêu gọi].
Thông tin sản phẩm được phép dùng: [dán thông số, giá, điều kiện khuyến mãi].
Hãy viết 5 biến thể CHỈ thay tiêu đề, giữ nguyên phần còn lại.
Mỗi biến thể dùng một góc khác nhau: (1) nêu vấn đề của khách, (2) nêu kết quả,
(3) đặt câu hỏi, (4) dùng lời khách thật bên dưới, (5) nêu con số cụ thể.
Ghi tên góc bên cạnh từng biến thể. Không dùng từ "nhất", "số một", "duy nhất",
không hứa kết quả, không thêm thông tin ngoài phần được phép dùng.
```

Ghi tên góc bên cạnh từng biến thể để sau khi có kết quả, bạn học được góc nào hợp với khách của mình chứ không chỉ biết câu nào thắng. Nếu thay cả tiêu đề lẫn hình lẫn lời kêu gọi cùng lúc, bạn sẽ không biết yếu tố nào tạo ra khác biệt.

## 6. Tổng hợp báo cáo chiến dịch từ file xuất

File xuất từ trình quản lý quảng cáo hay công cụ email thường là bảng CSV hoặc Excel vài trăm dòng. Có hai điều cần biết trước khi nhờ AI:

- Mô hình ngôn ngữ tính nhẩm trên bảng dán vào khung chat hay sai, nhất là cộng dồn nhiều dòng. Công cụ nào có chế độ phân tích file (tự viết và chạy mã để tính) thì đáng tin hơn, nhưng vẫn cần đối chiếu.
- Cách chắc nhất: bạn tự làm bảng tổng hợp (pivot) trong Excel hoặc Google Sheets, rồi đưa bảng đã tính xong cho AI viết phần diễn giải.

```text
Đây là bảng tổng hợp chiến dịch tháng 10 (tôi đã tự tính, coi các con số là đúng).
Hãy viết phần nhận xét cho báo cáo gửi quản lý, tối đa 200 chữ:
1. Ba điểm đáng chú ý nhất, mỗi điểm dẫn đúng con số trong bảng.
2. Nhóm quảng cáo nào có chi phí trên mỗi đơn cao bất thường so với trung bình.
3. Những câu hỏi tôi nên kiểm tra thêm trước khi kết luận.
Không tính lại số, không suy ra nguyên nhân nếu bảng không có dữ liệu chứng minh.
Phân biệt rõ "số liệu cho thấy" và "giả thuyết cần kiểm tra".

[Dán bảng]
```

Cách kiểm tra: mọi con số trong đoạn nhận xét phải tìm thấy trong bảng. Chú ý các câu giải thích nguyên nhân ("do nội dung hấp dẫn hơn"): AI rất hay viết những câu này dù không có dữ liệu nào chứng minh.

## Nhận ra và sửa văn AI

Những dấu hiệu dưới đây đã được cộng đồng biên tập Wikipedia tổng hợp khá kỹ cho tiếng Anh; trong tiếng Việt chúng hiện ra tương tự:

| Dấu hiệu | Ví dụ | Cách sửa |
| --- | --- | --- |
| Mở bài bằng bối cảnh lớn | "Trong nhịp sống hiện đại ngày nay..." | Xóa, vào thẳng vấn đề của khách |
| Chuỗi ba | "nhanh chóng, tiện lợi và hiệu quả" | Giữ một ý, thay bằng chi tiết đo được |
| Cặp phủ định | "không chỉ là... mà còn là..." | Nói thẳng nó là gì |
| Thổi phồng tầm quan trọng | "đánh dấu bước ngoặt", "nâng tầm trải nghiệm" | Thay bằng việc sản phẩm làm được |
| Kết bài kêu gọi chung chung | "Hãy cùng chúng tôi khám phá ngay hôm nay!" | Một hành động cụ thể, hoặc bỏ |
| Câu nào cũng dài bằng nhau | Đoạn văn đều tăm tắp | Chen câu ngắn, cắt câu thừa |

Dán danh sách cấm vào câu lệnh có giúp, nhưng chỉ chữa được bề mặt. Bài vẫn rỗng nếu không có gì riêng để nói. Một phép thử nhanh khi biên tập: thay tên thương hiệu của bạn bằng tên đối thủ. Nếu bài vẫn đúng thì bài đó chưa nói được gì về bạn. Cách chữa là thêm chất liệu chỉ bạn có: một câu khách nói thật, một con số của chính cửa hàng, một chi tiết sản xuất, một lần làm hỏng và sửa thế nào.

## Kiểm tra số liệu và tuyên bố về sản phẩm

Phần này không nên xem là thủ tục. AI viết quảng cáo có xu hướng tự thêm những thứ nghe thuyết phục: "được hàng nghìn khách tin dùng", "hiệu quả sau 7 ngày", "chiết xuất 100% tự nhiên". Nếu chúng không đúng, người chịu trách nhiệm là doanh nghiệp và người đăng, không phải công cụ.

Vài điểm mình chắc, và bạn vẫn nên đối chiếu văn bản gốc hoặc hỏi bộ phận pháp lý:

- Luật Quảng cáo yêu cầu nội dung quảng cáo trung thực, rõ ràng, không gây hiểu nhầm. Luật sửa đổi năm 2025 (số 75/2025/QH15, hiệu lực từ 01/01/2026) bổ sung trách nhiệm cho cả người trực tiếp quảng cáo, giới thiệu sản phẩm trên mạng.
- Luật Quảng cáo 2012 cấm dùng các từ "nhất", "duy nhất", "tốt nhất", "số một" khi không có tài liệu hợp pháp chứng minh. Mình chưa tự đọc toàn văn bản sửa đổi để biết câu chữ hiện hành, nên bạn hãy tra lại trước khi dùng các từ này.
- Thực phẩm chức năng, mỹ phẩm, thuốc, dịch vụ khám chữa bệnh, tài chính có quy định quảng cáo riêng và chặt hơn. Với các ngành này, đừng đăng bất kỳ câu nào về công dụng mà chưa qua người có trách nhiệm duyệt.

Danh sách dò trước khi đăng:

1. Mỗi con số trong bài có nguồn nội bộ nào? Không chỉ ra được thì xóa.
2. Mỗi công dụng có trong tài liệu sản phẩm đã được duyệt không?
3. Giá, thời hạn, điều kiện khuyến mãi có khớp với chương trình thật không?
4. Lời khách hàng trong bài có phải lời thật, và khách có đồng ý cho dùng không? Không để AI viết đánh giá giả.
5. Có so sánh với đối thủ không? Nếu có, bạn có bằng chứng không?

## Ranh giới dữ liệu

- Danh sách khách hàng, số điện thoại, email, lịch sử mua hàng là dữ liệu cá nhân. Luật Bảo vệ dữ liệu cá nhân (số 91/2025/QH15) có hiệu lực từ 01/01/2026. Chỉ đưa loại dữ liệu này vào công cụ mà công ty đã cho phép, và thường thì việc phân tích không cần đến nó: xóa cột định danh trước khi dán.
- Kế hoạch ra mắt chưa công bố, ngân sách, giá nhập: hỏi quy định của công ty trước khi dán vào bất kỳ công cụ nào.

## Nếu chỉ bắt đầu với một việc

Chọn việc 4. Lấy một bài gốc bạn tự viết, dán kèm hai bài cũ đúng giọng, nhờ AI cắt thành ba định dạng, rồi dò lại từng con số. Việc này ít rủi ro vì mọi ý đều đã do bạn duyệt từ bài gốc, và bạn thấy ngay chỗ AI hay "gọt" mất điều kiện hoặc thêm lời hứa. Khi đã quen tay kiểm tra thì mở sang các việc còn lại.

## Nguồn đọc thêm

- [7 điểm mới của Luật Quảng cáo sửa đổi 2025 (LuatVietnam)](https://luatvietnam.vn/linh-vuc-khac/diem-moi-cua-luat-quang-cao-sua-doi-2025-883-102813-article.html): tóm tắt trách nhiệm mới của người quảng cáo trên mạng. Đây là bài tổng hợp, khi cần chắc chắn hãy đọc toàn văn luật.
- [Những điểm mới trong Luật Bảo vệ dữ liệu cá nhân từ 1/1/2026 (VnEconomy)](https://vneconomy.vn/nhung-diem-moi-trong-luat-bao-ve-du-lieu-ca-nhan-tu-112026.htm): đọc để biết vì sao danh sách khách hàng không nên dán tùy tiện vào công cụ bên ngoài.
- [Signs of AI writing (Wikipedia, tiếng Anh)](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing): danh mục dấu hiệu văn AI do các biên tập viên Wikipedia tổng hợp, có ví dụ cụ thể cho từng loại.
