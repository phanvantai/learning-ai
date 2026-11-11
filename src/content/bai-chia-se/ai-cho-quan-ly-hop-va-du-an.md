---
title: "AI cho người quản lý nhóm: họp, biên bản, báo cáo tuần và giao việc"
date: 2026-11-10
tags: [Quản lý, Họp, Quy trình làm việc]
description: Sáu việc lặp lại của người quản lý nhóm hoặc quản lý dự án có thể làm cùng AI, mỗi việc có câu lệnh mẫu và cách kiểm tra, kèm ranh giới về ghi âm họp và dữ liệu nội bộ.
loai: ung-dung
---

Phần lớn thời gian của người quản lý không nằm ở việc ra quyết định, mà ở phần giấy tờ bao quanh quyết định: soạn chương trình họp, viết biên bản, gom tin nhắn thành báo cáo tuần, diễn đạt lại một yêu cầu cho đủ rõ để giao. Đây là phần AI giúp được nhiều, vì đầu vào là chữ và đầu ra cũng là chữ.

Phần AI không làm thay được là phần còn lại: chọn ưu tiên, đánh giá con người, chịu trách nhiệm về cam kết. Bài này đi qua sáu việc cụ thể, mỗi việc có câu lệnh mẫu dán được ngay và cách kiểm tra kết quả. Công cụ nào cũng được (ChatGPT, Gemini, Claude, Copilot), miễn là công ty bạn cho phép dùng với dữ liệu công việc.

## Việc nào giao AI, việc nào giữ lại

| Việc | Giao AI | Bạn giữ |
|---|---|---|
| Chuẩn bị họp | Soạn chương trình, gợi ý câu hỏi cần chốt | Quyết định họp để làm gì, mời ai |
| Biên bản họp | Sắp ghi chú thô thành việc, người, hạn | Xác nhận ai thật sự đã nhận việc gì |
| Báo cáo tuần | Gom dữ liệu rời rạc thành bản nháp | Chọn điều gì đáng báo, mức độ cảnh báo |
| Giao việc | Viết lại yêu cầu mơ hồ thành đầu việc rõ | Quyết định giao cho ai, ưu tiên ra sao |
| Phản hồi cho nhân viên | Sắp ý, gợi ý cách mở lời | Nhận định về con người, mọi kết luận |
| Rủi ro kế hoạch | Đóng vai người phản biện, liệt kê lý do có thể hỏng | Đánh giá rủi ro nào có thật, xử lý thế nào |

Quy tắc chung cho cả bảng: AI ra bản nháp, bạn là người ký tên. Với người đọc, biên bản hay báo cáo đó là của bạn, không phải của công cụ.

## 1. Chuẩn bị họp: bắt đầu từ điều cần chốt

Cuộc họp kéo dài thường do không ai viết ra trước là họp xong thì phải có gì. Thay vì nhờ AI "soạn agenda họp dự án", hãy đưa mục tiêu và để nó làm ngược từ đó: muốn chốt được các điểm này thì cần bàn gì, ai cần có mặt, ai cần chuẩn bị gì trước.

```text
Tôi là trưởng nhóm [loại nhóm], sắp có cuộc họp [thời lượng] phút với [những ai, vai trò gì].
Bối cảnh: [2-4 câu về tình hình hiện tại].
Sau cuộc họp tôi cần có: [liệt kê 1-3 kết quả, ví dụ "chốt ngày bàn giao", "ai phụ trách phần X"].

Hãy soạn:
1. Chương trình họp, mỗi mục gồm: nội dung, câu hỏi cần chốt, thời lượng, ai dẫn dắt.
2. Những thông tin từng người cần chuẩn bị trước khi vào họp.
3. Mục nào trong danh sách của tôi có thể giải quyết bằng tin nhắn, không cần họp.

Tổng thời lượng không vượt quá [thời lượng] phút. Nếu mục tiêu của tôi quá nhiều so với thời gian, hãy nói thẳng và đề xuất bỏ mục nào.
```

Dòng số 3 đáng giá hơn bạn nghĩ: một phần việc trong chương trình họp thường chỉ là thông báo, gửi trước bằng văn bản là đủ.

**Cách kiểm tra:** đọc từng "câu hỏi cần chốt" và tự hỏi có trả lời được bằng một quyết định cụ thể không. "Thảo luận về tiến độ" không phải câu hỏi. "Có lùi ngày bàn giao sang 20/11 không" mới là câu hỏi. Mục nào còn ở dạng đầu thì sửa tay hoặc yêu cầu AI viết lại.

## 2. Từ ghi chú họp thành biên bản có việc, người, hạn

Đầu vào có thể là ghi chú gõ vội của bạn hoặc bản chép lời từ công cụ ghi âm (xem phần lưu ý về ghi âm ở cuối bài). Điều cần ép AI làm là tách bạch ba thứ dễ bị trộn vào nhau: điều đã chốt, điều mới được nêu ra, và việc có người nhận.

```text
Dưới đây là ghi chú thô của cuộc họp [tên cuộc họp], ngày [ngày]. Người dự: [tên và vai trò].
Hãy viết biên bản theo đúng các mục sau:

1. Quyết định đã chốt: mỗi dòng một quyết định, ghi ai là người chốt.
2. Việc cần làm: bảng 3 cột Việc | Người phụ trách | Hạn. Thiếu thông tin nào thì ghi "chưa rõ", không tự điền.
3. Vấn đề được nêu nhưng chưa chốt.
4. Câu hỏi cần hỏi lại người dự để biên bản đầy đủ.

Chỉ dùng thông tin có trong ghi chú. Phân biệt rõ "A nói sẽ làm" với "A được đề nghị làm nhưng chưa nhận lời".

[Dán ghi chú hoặc bản chép lời]
```

**Cách kiểm tra:** lỗi nguy hiểm nhất của biên bản do AI viết là biến một câu lưng chừng thành cam kết. "Để em xem lại rồi báo" rất dễ trở thành "Lan phụ trách, hạn thứ Sáu". Vì vậy hãy đối chiếu từng dòng trong bảng việc với ghi chú gốc, và đếm số việc: nếu bạn nhớ cuộc họp có năm đầu việc mà bảng chỉ có bốn thì tìm việc bị sót. Với bản chép lời tự động, kiểm tra thêm tên người và con số, vì nhận dạng giọng nói tiếng Việt hay sai tên riêng, từ viết tắt và số liệu.

Một thói quen đáng giữ: gửi biên bản kèm câu "ai thấy sai phần của mình thì báo lại trong hôm nay". Người dự họp là lớp kiểm tra cuối cùng mà AI không thay được.

## 3. Báo cáo tuần từ dữ liệu rời rạc

Nguyên liệu của báo cáo tuần thường nằm ở nhiều chỗ: tin nhắn trong nhóm Zalo hoặc Teams, bảng công việc trên Google Sheets hay Trello, vài email. Việc gom lại mang tính cơ học, giao AI được. Việc quyết định điều gì đáng báo lên trên thì không.

Cách làm ổn định là giữ một khung báo cáo cố định và tuần nào cũng dùng lại:

```text
Tôi cần bản nháp báo cáo tuần cho [người nhận, ví dụ giám đốc khối], người này chỉ có 2 phút để đọc.
Nguồn dữ liệu gồm ba phần dán bên dưới: (A) bảng công việc xuất ra dạng văn bản, (B) tin nhắn nhóm trong tuần, (C) báo cáo tuần trước.

Viết theo khung:
1. Tóm tắt 3 dòng: tình trạng chung (đúng tiến độ / chậm / có rủi ro) và lý do.
2. Đã xong trong tuần.
3. Đang làm, kèm phần trăm hoặc mốc gần nhất nếu dữ liệu có.
4. Vướng mắc cần người nhận quyết định hoặc hỗ trợ, ghi rõ cần gì và trước ngày nào.
5. Kế hoạch tuần tới.

Quy tắc: mỗi ý ghi nguồn trong ngoặc (A, B hoặc C). Chỗ nào A và B mâu thuẫn nhau thì liệt kê riêng ở cuối, đừng tự chọn bên nào đúng. Không thêm nhận định mà dữ liệu không có.

(A) [dán]
(B) [dán]
(C) [dán]
```

Yêu cầu ghi nguồn giúp bạn kiểm tra nhanh: ý nào không gắn được nguồn là ý AI tự suy ra. Danh sách mâu thuẫn cũng hữu ích theo cách khác, vì nó chỉ ra chỗ bảng công việc chưa được cập nhật theo thực tế.

**Cách kiểm tra:** soát mục 1 và mục 4 kỹ nhất, vì đó là hai chỗ người nhận sẽ đọc và hành động. Tự hỏi: nếu sếp chỉ đọc ba dòng đầu, họ có hiểu đúng tình hình không? AI có xu hướng viết giọng trung tính, làm nhẹ đi một vấn đề lẽ ra phải báo động. Mức độ cảnh báo là thứ bạn phải tự chỉnh.

Lưu ý khi dán tin nhắn nhóm: bỏ các đoạn chuyện riêng, thông tin cá nhân của thành viên, và mọi thứ nằm ngoài phạm vi công ty cho phép đưa vào công cụ AI.

## 4. Viết lại yêu cầu mơ hồ thành đầu việc rõ tiêu chí hoàn thành

"Em làm lại cái slide cho đẹp hơn nhé" là kiểu giao việc tạo ra ba vòng sửa. Một đầu việc rõ cần ít nhất: kết quả bàn giao là gì, thế nào là xong, hạn, và phạm vi không cần làm.

Ở việc này, giá trị của AI nằm ở chỗ nó hỏi ngược lại bạn. Hãy yêu cầu nó hỏi trước khi viết:

```text
Tôi sắp giao một việc cho thành viên trong nhóm. Yêu cầu ban đầu của tôi như sau:
"[dán yêu cầu thô]"

Bước 1: Hãy đóng vai người nhận việc, mới vào nhóm 3 tháng. Liệt kê tối đa 7 câu hỏi mà người đó cần được trả lời để làm đúng ngay lần đầu. Dừng lại chờ tôi trả lời.

Bước 2: Sau khi tôi trả lời, viết lại thành một đầu việc gồm:
- Mục đích (việc này phục vụ điều gì)
- Kết quả bàn giao (file gì, định dạng nào, gửi cho ai)
- Tiêu chí hoàn thành: 3-5 dòng kiểm tra được bằng có/không
- Ngoài phạm vi (những gì không cần làm)
- Hạn và mốc kiểm tra giữa chừng
```

**Cách kiểm tra:** đọc từng tiêu chí hoàn thành và hỏi "hai người khác nhau đọc dòng này có kết luận giống nhau không". "Slide chuyên nghiệp hơn" thì không. "Mỗi slide một ý chính, không quá 30 chữ, số liệu có ghi nguồn" thì có. Những câu hỏi ở bước 1 mà bạn không trả lời được là dấu hiệu chính bạn chưa rõ mình muốn gì, và nên làm rõ trước khi giao.

## 5. Chuẩn bị phản hồi 1-1 cho nhân viên

Đây là việc cần ranh giới chặt nhất trong bài. AI không quan sát nhân viên của bạn, không biết hoàn cảnh của họ, và mọi "nhận xét" nó đưa ra chỉ dựa trên vài dòng bạn mô tả. Dùng nó để sắp ý của bạn cho mạch lạc và tìm cách nói dễ nghe hơn. Đừng dùng nó để kết luận về một con người.

Một khung phổ biến cho phản hồi là tình huống, hành vi, tác động: chuyện xảy ra khi nào, người đó đã làm gì (quan sát được), và việc đó gây ra điều gì. Khung này buộc bạn nói về sự việc thay vì dán nhãn tính cách.

```text
Tôi sắp có buổi trao đổi riêng với một thành viên trong nhóm. Dưới đây là các quan sát của tôi, đã bỏ tên và chi tiết nhận dạng:
[liệt kê quan sát: việc gì, khi nào, kết quả ra sao]

Hãy giúp tôi:
1. Sắp các quan sát theo khung tình huống, hành vi, tác động. Quan sát nào của tôi thực ra là phán xét (ví dụ "thiếu trách nhiệm") thì chỉ ra và hỏi tôi sự việc cụ thể đằng sau.
2. Gợi ý 3 câu hỏi mở để tôi nghe phía bạn ấy trước khi nói nhận định của mình.
3. Gợi ý cách mở đầu buổi trao đổi trong 2-3 câu, giọng thẳng thắn và tôn trọng.

Không đánh giá người này, không suy đoán nguyên nhân hay động cơ. Chỉ làm việc trên những gì tôi viết.
```

**Cách kiểm tra:** với mỗi ý trong bản AI trả về, tự hỏi "mình có ví dụ cụ thể, tự mình thấy hoặc có bằng chứng không". Ý nào không có thì bỏ. Rồi viết lại bằng lời của chính bạn: một đoạn phản hồi nghe như văn mẫu sẽ làm buổi nói chuyện mất tin cậy.

**Không đưa vào AI:** tên đầy đủ, lương, kết quả đánh giá chính thức, chuyện sức khỏe hay gia đình của nhân viên. Và không dùng AI để chấm điểm, xếp loại hay đề xuất kỷ luật. Quyết định đó ảnh hưởng tới thu nhập và sự nghiệp của một người, nên phải do người chịu trách nhiệm đưa ra, theo quy trình nhân sự của công ty.

## 6. Tìm rủi ro kế hoạch bằng pre-mortem

Pre-mortem (tạm dịch: mổ xẻ trước khi hỏng) là kỹ thuật do nhà tâm lý học Gary Klein giới thiệu trên Harvard Business Review năm 2007. Thay vì hỏi "kế hoạch này có rủi ro gì", cả nhóm giả định rằng kế hoạch đã thất bại rồi, sau đó liệt kê các lý do hợp lý dẫn tới thất bại đó. Theo Klein, cách đặt vấn đề này giúp người có lo ngại dễ nói ra hơn trong giai đoạn lập kế hoạch.

AI hợp với vai người phản biện vì nó không ngại sếp và không mệt. Giới hạn cũng rõ: nó không biết nội tình công ty bạn, nên sẽ đưa ra cả rủi ro chung chung lẫn rủi ro trúng. Bạn bù lại bằng cách cho nó càng nhiều bối cảnh thật càng tốt.

```text
Dưới đây là kế hoạch dự án của tôi: [dán kế hoạch: mục tiêu, mốc thời gian, nhân sự, phụ thuộc bên ngoài].

Hãy đóng vai một người phản biện khó tính. Giả sử 6 tháng sau, dự án này đã thất bại: trễ hạn nặng hoặc không đạt mục tiêu.
1. Viết 8-10 lý do có khả năng nhất dẫn tới thất bại, mỗi lý do gắn với một chi tiết cụ thể trong kế hoạch (trích lại chi tiết đó).
2. Với mỗi lý do: dấu hiệu cảnh báo sớm mà tôi có thể nhìn thấy trong 2-4 tuần đầu.
3. Liệt kê những giả định mà kế hoạch đang ngầm dựa vào nhưng không viết ra.
4. Kế hoạch còn thiếu thông tin gì khiến bạn không đánh giá được?

Không khen kế hoạch. Không đưa lý do chung chung áp dụng được cho mọi dự án.
```

**Cách kiểm tra:** với danh sách này bạn không kiểm tra đúng sai, mà lọc. Gạch những lý do không gắn với tình hình thật, giữ lại ba đến năm cái khiến bạn chột dạ. Sau đó mang danh sách đã lọc vào cuộc họp nhóm và cho mọi người bổ sung. Bản của AI chỉ là mồi để nhóm bắt đầu nói, không thay cho pre-mortem với những người thật sự làm dự án.

Mục số 3 thường là mục có ích nhất. Ví dụ, một kế hoạch có thể đang ngầm giả định rằng bên cung cấp trả lời trong hai ngày, hoặc không ai trong nhóm nghỉ phép vào tháng cao điểm.

## Ghi âm họp và dữ liệu nội bộ

Nhiều công cụ họp trực tuyến và ứng dụng ghi chú hiện có tính năng ghi âm rồi chép lời tự động. Trước khi bật, có ba việc cần làm:

- **Xin phép trước, nói rõ mục đích.** Thông báo ngay đầu cuộc họp rằng bạn ghi âm để làm biên bản, ai sẽ được xem bản ghi, và giữ trong bao lâu. Có người không đồng ý thì quay về ghi chú tay. Giọng nói và lời phát biểu là dữ liệu cá nhân của người dự, và với khách hàng hay đối tác thì có thể còn bị ràng buộc bởi thỏa thuận bảo mật. Mình không phải người làm luật, nên nếu cuộc họp có yếu tố pháp lý thì hãy hỏi bộ phận pháp chế của công ty.
- **Hỏi công ty được phép dùng công cụ nào.** Bản dùng cho doanh nghiệp và bản miễn phí cho cá nhân thường có điều khoản xử lý dữ liệu khác nhau. Đừng tự đoán, hãy đọc điều khoản hoặc hỏi bộ phận IT.
- **Lọc trước khi dán.** Lương, đánh giá nhân sự, số liệu tài chính chưa công bố, thông tin khách hàng: bỏ ra hoặc thay bằng ký hiệu (Nhân viên A, Khách hàng X) trước khi đưa vào AI.

Thêm một điểm ít người để ý: khi biết mình bị ghi âm, người ta nói dè dặt hơn. Với buổi trao đổi riêng hoặc cuộc họp cần nói thẳng, không ghi âm thường là lựa chọn tốt hơn.

## Lỗi thường gặp

- **Gửi thẳng bản AI viết.** Biên bản có một dòng gán sai người là đủ để lần sau không ai tin biên bản của bạn. Luôn có một lượt đối chiếu với ghi chú gốc.
- **Đưa quá ít bối cảnh.** AI không biết dự án của bạn. Câu lệnh ba dòng cho ra kết quả đúng với mọi dự án, tức là không đúng riêng với dự án nào.
- **Để AI quyết thay.** "Nên giao việc này cho ai" hay "nhân viên này có đạt không" là câu hỏi của bạn. AI có thể liệt kê tiêu chí để bạn cân nhắc, không hơn.
- **Mỗi lần gõ lại từ đầu.** Sáu câu lệnh trên nên được lưu thành mẫu, điền sẵn phần cố định (vai trò của bạn, người nhận báo cáo, khung biên bản của công ty) để mỗi tuần chỉ thay phần dữ liệu.

Nếu mới bắt đầu, mình gợi ý thử việc số 2 trước. Biên bản họp có đầu vào rõ, có người dự họp kiểm tra lại giúp bạn, và sai thì sửa được ngay trong ngày.

## Nguồn đọc thêm

- [Performing a Project Premortem](https://hbr.org/2007/09/performing-a-project-premortem), Gary Klein, Harvard Business Review, 9/2007. Bài gốc giới thiệu pre-mortem, ngắn, nên đọc trước khi thử với nhóm.
- [Pre-mortem trên trang của Gary Klein](https://www.gary-klein.com/premortem). Mô tả gọn các bước chạy một buổi pre-mortem với nhóm.
