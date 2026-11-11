---
title: "Kế toán dùng AI: nhờ viết công thức, đừng nhờ cộng số"
date: 2026-11-09
tags: [Kế toán, Quy trình làm việc, Prompt mẫu]
description: Sáu việc lặp lại của kế toán doanh nghiệp nhỏ mà AI giúp được (công thức Excel, đối chiếu sao kê, soát bảng kê, email công nợ, đọc văn bản thuế, diễn giải bút toán), kèm câu lệnh mẫu, cách kiểm tra và những việc phải giữ lại.
loai: ung-dung
---

Mình là kỹ sư phần mềm, không phải kế toán. Phần mình nắm chắc là AI làm tốt việc gì và hỏng theo kiểu nào. Phần nghiệp vụ, bài chỉ chọn những việc phổ biến ở phòng kế toán doanh nghiệp nhỏ và vừa, và cố ý không nêu thuế suất, hạn nộp hay số hiệu văn bản nào. Lý do nằm ở nguyên tắc thứ hai bên dưới.

## Ba nguyên tắc trước khi mở khung chat

### 1. Đừng nhờ AI tự cộng, hãy nhờ nó viết công thức

Mô hình ngôn ngữ sinh ra chữ tiếp theo dựa trên xác suất, nó không có máy tính bên trong. Dán một bảng 300 dòng rồi hỏi "tổng cột tiền thuế là bao nhiêu", bạn có thể nhận một con số trông hợp lý nhưng lệch vài nghìn đồng hoặc sót vài dòng, và nó không báo lỗi.

Cách dùng đúng: mô tả bảng cho AI, nhờ nó viết công thức hoặc các bước Power Query, rồi để Excel tính. Excel tính sai thì bạn lần lại được công thức. AI nhẩm sai thì không có gì để lần. Một số công cụ cho tải file lên và tự chạy code để tính; cách này chính xác hơn nhẩm, nhưng bạn khó thấy nó đã lọc dòng nào, nên vẫn phải so tổng với con số tự tính bằng Excel.

### 2. Không tin AI về thuế suất, hạn nộp, số hiệu văn bản

AI học từ dữ liệu cũ, trong khi quy định thuế đổi thường xuyên. Nó có thể trả lời trôi chảy theo một quy định đã hết hiệu lực, hoặc ghép số hiệu của văn bản này với nội dung của văn bản khác. Mọi con số và căn cứ pháp lý AI đưa ra chỉ là gợi ý để đi tra: mở văn bản gốc trên cổng thông tin chính thức, đọc đúng điều khoản đó và kiểm văn bản còn hiệu lực hay không.

### 3. Ẩn danh dữ liệu trước khi dán

Sổ sách chứa đúng những thứ không nên rời khỏi công ty: tên khách hàng, mã số thuế, số tài khoản, lương từng người. Trước khi dán:

- Hỏi công ty cho phép dùng công cụ nào. Cam kết về dữ liệu khác nhau giữa các công cụ và các gói.
- Thay tên bằng mã (KH001, NCC014), xóa cột mã số thuế, số tài khoản, số điện thoại nếu việc đang làm không cần tới.
- Với việc viết công thức, AI chỉ cần tên cột và 3 đến 5 dòng mẫu do bạn tự đặt ra. Không cần dán dữ liệu thật.

## Việc 1: Viết và giải thích công thức Excel, Power Query

Việc này rủi ro thấp nhất trong danh sách vì kết quả kiểm được ngay.

```text
Tôi dùng Excel [phiên bản, ví dụ Microsoft 365], dấu ngăn cách tham số trong công thức là dấu chấm phẩy (;).
Sheet "SoChiTiet" có các cột: A = Ngày chứng từ, B = Số chứng từ, C = Mã khách hàng,
D = Diễn giải, E = Phát sinh Nợ, F = Phát sinh Có.
Sheet "TongHop" có cột A = Mã khách hàng.

Tôi cần ở sheet "TongHop": cột B = tổng phát sinh Nợ, cột C = tổng phát sinh Có của từng mã
khách hàng, chỉ tính chứng từ trong khoảng ngày nhập ở ô F1 (từ ngày) và F2 (đến ngày).

Hãy viết công thức cho ô B2 và C2, giải thích từng tham số bằng tiếng Việt,
và nêu 3 trường hợp công thức có thể cho kết quả sai (ví dụ số bị lưu dạng chữ).
```

Hai chi tiết trong câu lệnh hay bị bỏ quên. Phiên bản Excel quyết định hàm nào dùng được (XLOOKUP không có ở các bản cũ). Dấu ngăn cách tham số phụ thuộc thiết lập vùng của máy: nhiều máy ở Việt Nam dùng dấu chấm phẩy, AI thì mặc định viết dấu phẩy, dán vào là Excel báo lỗi.

Chiều ngược lại cũng đáng dùng: dán một công thức lồng năm sáu tầng do người cũ để lại và hỏi "công thức này làm gì, từng phần một, và nó sẽ sai khi nào".

**Cách kiểm tra:** chọn hai ba mã khách hàng, lọc tay trên sổ chi tiết rồi so với kết quả công thức. Thử thêm một mã không có phát sinh và một mã có phát sinh nằm đúng ngày đầu và ngày cuối kỳ, vì lỗi hay nằm ở biên.

## Việc 2: Đối chiếu sao kê, công nợ với sổ

Ví dụ: cuối tháng bạn có file sao kê ngân hàng và sổ tiền gửi xuất từ phần mềm kế toán, cần tìm giao dịch có bên này mà thiếu bên kia. Theo nguyên tắc 1, nhờ AI thiết kế cách so khớp chứ đừng để nó tự so.

```text
Tôi cần đối chiếu hai bảng trong Excel:
- Bảng "SaoKe": Ngày giao dịch, Số tiền ghi Nợ, Số tiền ghi Có, Nội dung.
- Bảng "SoTienGui": Ngày chứng từ, Số chứng từ, Thu, Chi, Diễn giải.

Mục tiêu: liệt kê (1) dòng có ở sao kê mà không có ở sổ, (2) dòng có ở sổ mà không có ở sao kê.
Hai bảng không có mã chung. Hãy:
1. Đề xuất cách tạo khóa so khớp và nói rõ điểm yếu của cách đó.
2. Hướng dẫn từng bước bằng Power Query để ra hai danh sách chênh lệch.
3. Liệt kê các bước làm sạch dữ liệu cần làm trước (khoảng trắng thừa, số dạng chữ, định dạng ngày).
4. Cách xử lý khi có hai giao dịch cùng ngày cùng số tiền.
```

Trong Power Query, phép cần dùng là Merge với kiểu nối "Left Anti": giữ lại những dòng ở bảng bên trái không tìm thấy dòng tương ứng ở bảng bên phải ([tài liệu của Microsoft về left anti join](https://learn.microsoft.com/en-us/power-query/merge-queries-left-anti)). Làm hai lần, đổi vai hai bảng, là ra hai danh sách chênh lệch.

Đừng bỏ bước làm sạch ở mục 3: khoảng trắng thừa, số tiền lưu dạng chữ, ngày bị đảo tháng với ngày khiến hai dòng giống nhau mà không khớp.

Những khoản không khớp một-một thì AI không ghép thay bạn được: một lần chuyển khoản trả gộp ba hóa đơn, phí ngân hàng bị trừ thẳng vào số tiền nhận, khách chuyển thiếu vài nghìn. Danh sách chênh lệch chỉ thu hẹp phạm vi.

**Cách kiểm tra:** số dư đầu kỳ cộng phát sinh, sau khi điều chỉnh các khoản chênh lệch đã giải thích, phải ra đúng số dư cuối kỳ trên sao kê. Nếu không ra, cách so khớp đang bỏ sót hoặc ghép nhầm. Nhóm giao dịch trùng ngày trùng tiền dễ bị ghép nhầm nhất, nên xem tay.

## Việc 3: Soát bảng kê tìm dòng bất thường

Với bảng kê vài trăm dòng, hỏi AI "bảng này có gì lạ không" cho kết quả hên xui: nó chỉ ra vài dòng, bỏ qua vài dòng khác. Cách ổn định hơn là nhờ AI nghĩ ra các phép kiểm tra rồi biến chúng thành cột công thức.

```text
Bảng kê hóa đơn mua vào trong Excel có các cột: A = Ký hiệu hóa đơn, B = Số hóa đơn,
C = Ngày hóa đơn, D = Mã số thuế người bán, E = Tiền hàng chưa thuế, F = Thuế suất,
G = Tiền thuế, H = Tổng thanh toán.

Hãy đề xuất danh sách phép kiểm tra để phát hiện dòng có khả năng nhập sai, và với mỗi phép
viết một công thức Excel (dấu ngăn cách ;) trả về TRUE khi dòng đó cần xem lại.
Ví dụ các hướng: hóa đơn trùng, tiền thuế không khớp với tiền hàng nhân thuế suất,
tổng thanh toán không bằng tiền hàng cộng tiền thuế, ngày nằm ngoài kỳ kê khai,
mã số thuế có độ dài khác thường.
Không tự tính trên dữ liệu, chỉ viết công thức.
```

Kéo công thức xuống, lọc TRUE là có danh sách cần xem. Phép kiểm tra chạy trên mọi dòng, và tháng sau dùng lại được.

AI sẽ tự chọn vài tham số mà nó không có căn cứ để chọn. Ba chỗ bạn phải quyết:

- **Làm tròn và mức lệch cho phép.** Lấy theo cách phần mềm kế toán của bạn đang tính và quy định hiện hành, đừng lấy theo con số AI gợi ý.
- **Thế nào là hóa đơn trùng.** AI thường đề xuất ghép vài cột làm khóa (ví dụ mã số thuế người bán, ký hiệu, số hóa đơn). Tổ hợp nào đủ để coi hai dòng là một hóa đơn thì đối chiếu với quy định về hóa đơn đang áp dụng.
- **Dòng không theo khuôn.** Thuế suất ghi bằng chữ (ví dụ hàng không chịu thuế), hóa đơn điều chỉnh hay thay thế cần xử lý riêng, nếu không sẽ báo động giả.

**Cách kiểm tra:** cố ý sửa sai hai ba dòng (đổi một số tiền thuế, nhân đôi một hóa đơn) rồi xem cột kiểm tra có bắt được không. Các phép này chỉ bắt lỗi nhập liệu. Hóa đơn có hợp lệ hay không, chi phí có được trừ hay không là phán đoán nghề của bạn, và tình trạng hóa đơn thì tra trên cổng hóa đơn điện tử của cơ quan thuế chứ không hỏi AI.

## Việc 4: Email nhắc công nợ và bản nháp giải trình

Nhắc nợ khó ở giọng: đủ rõ để khách hiểu là cần trả, đủ mềm để còn làm ăn tiếp. AI lo phần giọng, mọi con số do bạn đưa vào.

```text
Viết email nhắc thanh toán gửi khách hàng doanh nghiệp, xưng "em", gọi "anh/chị".
Thông tin (chỉ dùng đúng các thông tin này, không thêm số liệu hay điều khoản nào khác):
- Hóa đơn số [..] ngày [..], số tiền [..], hạn thanh toán theo hợp đồng [..].
- Đã quá hạn [..] ngày. Đây là lần nhắc thứ [1/2/3].
- Quan hệ: khách lâu năm, trước giờ thanh toán đúng hạn.

Viết 2 phiên bản: (a) nhẹ nhàng, giả định bên khách sót việc; (b) dứt khoát hơn, đề nghị
xác nhận ngày thanh toán cụ thể. Mỗi bản dưới 120 chữ, có dòng tiêu đề.
Không nhắc tới phạt chậm trả hay biện pháp pháp lý.
```

Không có câu cấm ở cuối, AI dễ tự thêm "theo hợp đồng, quý công ty sẽ chịu lãi chậm trả" dù hợp đồng của bạn không có điều khoản đó. Kiểu lỗi tự thêm cam kết này được nói kỹ hơn ở bài [AI trong bán hàng và chăm sóc khách hàng](/bai-chia-se/ai-cho-sales-cham-soc-khach-hang/).

Với công văn giải trình gửi cơ quan thuế hay ngân hàng, bạn liệt kê sự việc theo thời gian và chứng từ đang có, nhờ AI dựng thành văn bản mạch lạc và chỉ ra chỗ người đọc sẽ hỏi lại. Lý do phải là của bạn, căn cứ pháp lý để trống tới khi tra văn bản gốc.

**Cách kiểm tra:** so từng con số, ngày tháng, số hóa đơn trong email với sổ. Đọc thêm một lượt để tìm câu nào chứa cam kết hoặc quy định mà bạn không đưa vào.

## Việc 5: Đọc văn bản thuế để biết cần đọc kỹ điều nào

Một thông tư mới có thể dài vài chục trang, phần lớn không liên quan tới công ty bạn. AI khoanh vùng giúp được, với điều kiện bạn dán nguyên văn văn bản (hoặc tải file lên) chứ không hỏi theo trí nhớ của nó.

```text
Dưới đây là toàn văn một văn bản tôi lấy từ cổng thông tin chính thức.
Công ty tôi: [ngành, quy mô, phương pháp tính thuế, có/không xuất nhập khẩu...].

Chỉ dựa vào văn bản tôi dán, hãy:
1. Liệt kê các điều, khoản có khả năng ảnh hưởng tới công ty như trên. Mỗi dòng ghi số điều,
   số khoản, và trích nguyên văn câu then chốt trong ngoặc kép.
2. Liệt kê các điều tôi có thể bỏ qua và lý do.
3. Chỗ nào văn bản dẫn chiếu sang văn bản khác mà tôi chưa dán, ghi rõ để tôi tìm đọc.
Không diễn giải thêm ngoài văn bản. Không chắc thì ghi "không chắc".

[Dán văn bản]
```

Yêu cầu trích nguyên văn là chốt chặn chính: tìm câu đó trong văn bản gốc, không thấy nghĩa là AI đã bịa. Mục 3 cần vì văn bản thuế hay sửa đổi, bổ sung một văn bản trước đó, đọc riêng một tờ thì thiếu.

**Cách kiểm tra:** đầu ra là danh sách điều cần đọc, không phải kết luận. Bạn vẫn đọc các điều đó trong văn bản gốc, và lướt cả mục lục để chắc AI không xếp nhầm một điều liên quan vào nhóm "bỏ qua".

## Việc 6: Chuẩn hóa diễn giải bút toán

Sổ có ba người nhập thì có ba kiểu diễn giải: "thu tien kh A", "KH A thanh toán HĐ 125", "A ck". Tìm kiếm và lọc về sau rất khổ.

```text
Mẫu diễn giải chuẩn của công ty tôi:
- Thu tiền bán hàng: "Thu tiền [mã KH] theo HĐ [số hóa đơn] ngày [dd/mm/yyyy]"
- Chi trả nhà cung cấp: "Chi trả [mã NCC] theo HĐ [số hóa đơn] ngày [dd/mm/yyyy]"
- [các mẫu khác của bạn]

Dưới đây là 40 dòng diễn giải cũ. Trả về bảng 3 cột: diễn giải gốc, diễn giải chuẩn hóa, ghi chú.
Nếu dòng gốc thiếu thông tin để điền mẫu (ví dụ không có số hóa đơn), để trống phần đó dạng
[thiếu số HĐ] và ghi vào cột ghi chú. Tuyệt đối không tự điền thông tin không có trong dòng gốc.
```

Làm theo lô nhỏ vài chục dòng. Lô dài, AI dễ bỏ sót dòng hoặc bắt đầu tự điền ở nửa cuối.

**Cách kiểm tra:** đếm số dòng trả về bằng số dòng gửi đi. Lọc cột ghi chú để xử lý tay các dòng thiếu. Rà riêng số hóa đơn và ngày, vì đây là chỗ AI hay "sửa giúp" một ký tự. Chỉ sửa diễn giải, không để AI đụng tới tài khoản hạch toán hay số tiền.

## Việc nào giao, việc nào giữ

| Việc | Giao AI | Bạn giữ |
|---|---|---|
| Công thức, Power Query | Viết, giải thích, tìm lỗi công thức | Thử trên mẫu đã biết đáp án |
| Đối chiếu | Thiết kế cách so khớp, các bước làm sạch | Ghép các khoản không khớp một-một, ký biên bản |
| Soát bảng kê | Nghĩ phép kiểm tra, viết công thức | Quy tắc làm tròn, khóa trùng, phán đoán hóa đơn hợp lệ |
| Email, giải trình | Bản nháp, chỉnh giọng | Số liệu, lý do, căn cứ, quyết định gửi |
| Văn bản thuế | Khoanh vùng điều cần đọc | Đọc văn bản gốc, kết luận áp dụng |
| Diễn giải | Viết lại theo mẫu | Định khoản, số tiền |

Hai việc không nên giao, kể cả khi AI tỏ ra làm được:

- **Định khoản và xác định nghĩa vụ thuế.** Người chịu trách nhiệm với cơ quan thuế là bạn và công ty.
- **Lập báo cáo tài chính, tờ khai để nộp thẳng.** AI có thể giúp soát lỗi trình bày, không thay người lập và người ký.

## Bắt đầu từ đâu

Nếu chỉ thử một việc trong tuần này, mình đề nghị việc 3: bộ cột kiểm tra chạy lại được mỗi kỳ mà không cần mở AI nữa.

Sáu câu lệnh trên phải sửa cho khớp tên cột và mẫu biểu của công ty bạn. Sửa xong thì lưu lại theo cách ở bài [Từ hỏi từng câu đến một hệ thống làm việc với AI của riêng bạn](/bai-chia-se/xay-he-thong-ai-ca-nhan/). Sang việc mới thì mở cuộc trò chuyện mới và mô tả lại bảng; lý do nằm ở thói quen "chia ca" trong bài [Cùng một AI, vì sao người khác dùng hiệu quả hơn bạn?](/bai-chia-se/cung-ai-nguoi-dung-hieu-qua-hon/).

## Nguồn đọc thêm

- [Left anti join (Microsoft Learn)](https://learn.microsoft.com/en-us/power-query/merge-queries-left-anti): ví dụ từng bước cho phép nối dùng ở việc 2, có hình minh họa kết quả.
