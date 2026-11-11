---
title: "Tuyển dụng và nhân sự: việc nào AI viết giúp, vì sao không để AI loại CV"
date: 2026-11-10
tags: [Nhân sự, Quy trình làm việc, Prompt mẫu]
description: Sáu việc lặp lại của người làm nhân sự, tuyển dụng mà AI giúp được, kèm câu lệnh mẫu và cách kiểm tra. Phần dài nhất dành cho việc dễ sai nhất là đưa CV vào AI.
loai: ung-dung
---

Nhìn từ góc AI, công việc nhân sự có hai nửa. Một nửa là viết: JD, câu hỏi phỏng vấn, thư từ, tài liệu hướng dẫn. Nửa này AI làm nhanh và rủi ro thấp. Nửa kia là đánh giá con người dựa trên dữ liệu cá nhân của họ, nơi AI có thể gây hại thật và người chịu hậu quả là ứng viên.

Mình là kỹ sư phần mềm, không làm nhân sự. Bài viết từ phía hiểu AI hỏng ở đâu; phần nghiệp vụ và pháp lý bạn đối chiếu với thực tế công ty và văn bản gốc.

## Việc 1: Viết JD từ ghi chú của trưởng bộ phận

Trưởng bộ phận thường nhắn vài dòng qua Zalo: "cần 1 bạn kế toán tổng hợp, 3 năm kinh nghiệm, biết phần mềm, chịu được áp lực cuối tháng". AI viết JD từ mấy dòng đó rất trôi, và đó chính là vấn đề: nó lấp chỗ trống bằng những câu JD nào cũng có ("năng động, chịu được áp lực cao, có tinh thần trách nhiệm"). Nên bắt nó hỏi lại trước khi viết.

```text
Dưới đây là ghi chú của trưởng bộ phận về vị trí cần tuyển.
Bước 1: Chưa viết JD. Hãy liệt kê những thông tin còn thiếu để viết một JD cụ thể
(ví dụ: báo cáo cho ai, 3 đầu việc chiếm nhiều thời gian nhất, công cụ đang dùng,
tiêu chí nào bắt buộc và tiêu chí nào chỉ là điểm cộng). Tối đa 8 câu hỏi.
Bước 2: Sau khi tôi trả lời, viết JD gồm: mô tả công việc (5-7 gạch đầu dòng, bắt đầu
bằng động từ), yêu cầu bắt buộc (tối đa 5), điểm cộng (tối đa 3), quyền lợi.
Phần quyền lợi và mức lương chỉ dùng đúng thông tin tôi cung cấp, thiếu thì để [cần bổ sung].
Không đưa yêu cầu về tuổi, giới tính, ngoại hình, tình trạng hôn nhân, quê quán.

Ghi chú: [dán ghi chú]
```

Các câu hỏi ở bước 1 bạn chuyển lại cho trưởng bộ phận.

Dòng cuối của câu lệnh có lý do. Nhiều JD vẫn ghi "nam, dưới 30 tuổi" theo thói quen, trong khi pháp luật lao động có quy định cấm phân biệt đối xử trong lao động (phạm vi cụ thể bạn đối chiếu Bộ luật Lao động hiện hành). Nếu ghi chú có yêu cầu kiểu đó, AI sẽ chép lại nguyên xi trừ khi bạn dặn trước.

**Cách kiểm tra:** gạch mọi câu mà đặt vào JD của vị trí khác vẫn đúng. Gửi trưởng bộ phận xác nhận phần yêu cầu bắt buộc, vì AI hay nâng "biết thì tốt" thành "bắt buộc". Soát phần quyền lợi từng dòng: đây là lời hứa của công ty với ứng viên.

## Việc 2: Bộ câu hỏi phỏng vấn theo năng lực và phiếu chấm

Phỏng vấn mỗi người hỏi một kiểu thì không so sánh được ứng viên. Phỏng vấn có cấu trúc dùng cùng một bộ câu hỏi hành vi cho mọi ứng viên, chấm theo cùng một thang. Soạn bộ này tốn công, và AI làm tốt.

```text
Vị trí: [tên]. Ba năng lực cần đánh giá: [ví dụ: cẩn thận với số liệu, phối hợp với
bộ phận khác, xử lý khi bị dồn việc cuối kỳ].
Với mỗi năng lực, hãy soạn:
1. Hai câu hỏi hành vi dạng "Hãy kể về một lần anh/chị..." (hỏi việc đã làm, không hỏi giả định).
2. Hai câu hỏi đào sâu để dùng khi ứng viên trả lời chung chung.
3. Mô tả câu trả lời ở mức 1, 3 và 5 điểm, bằng hành vi quan sát được, không dùng tính từ
   kiểu "tốt", "khá".
Sau đó dựng phiếu chấm dạng bảng: năng lực, điểm 1-5, bằng chứng (ghi lại ứng viên đã nói gì, không ghi cảm giác).
Không soạn câu hỏi về tuổi, hôn nhân, kế hoạch sinh con, tôn giáo, quê quán.
```

**Cách kiểm tra:** đưa trưởng bộ phận đọc mô tả mức 1, 3, 5 và hỏi "người giỏi nhất team mình có trả lời như mức 5 này không", vì AI mô tả mức điểm theo sách vở. Dùng thử với một hai ứng viên đầu rồi sửa những câu ai cũng trả lời giống nhau.

## Việc 3: Tóm tắt CV theo tiêu chí, và vì sao không để AI loại ứng viên

Đọc 150 CV cho một vị trí rất mệt, nên đây là việc hấp dẫn nhất, và cũng cần dè dặt nhất.

### Rủi ro thứ nhất: thiên kiến

Mô hình ngôn ngữ học từ văn bản do con người viết, nên mang theo định kiến có trong đó. Khi bạn hỏi "ứng viên này có phù hợp không", câu trả lời có thể bị kéo bởi những thứ không liên quan tới năng lực: tên gợi ra giới tính, năm sinh, tên trường, quê quán, hay chỉ là CV viết trôi chảy hơn. Mức độ lệch tùy mô hình và cách hỏi, bài này không có con số đáng tin để đưa ra. Bạn cũng không biết nó đã cân những yếu tố nào: phần "lý do" nó viết ra cũng là văn bản được sinh ra, chưa chắc phản ánh cái thật sự dẫn tới điểm số.

Cùng một CV, hỏi hai lần còn có thể ra hai điểm khác nhau, và thứ tự dán các CV cũng có thể ảnh hưởng tới kết quả. Một quy trình loại người không lặp lại được thì không công bằng.

### Rủi ro thứ hai: dữ liệu cá nhân

Dán CV vào một công cụ AI là chuyển họ tên, ngày sinh, số điện thoại, địa chỉ, ảnh của ứng viên cho bên thứ ba, trong khi họ nộp hồ sơ cho công ty bạn chứ không phải cho nhà cung cấp AI.

Việt Nam đã có Luật Bảo vệ dữ liệu cá nhân (Luật số 91/2025/QH15), có hiệu lực từ 1/1/2026. Phần dưới đây dựa trên bản tin báo chí và các bản tóm tắt pháp lý chứ không phải toàn văn luật, nên bạn coi là đầu mối để tra. Theo [bản tin của báo Tiền Phong](https://tienphong.vn/doanh-nghiep-khi-cham-dut-hop-dong-phai-xoa-du-lieu-ca-nhan-nguoi-lao-dong-post1764533.tpo), Điều 25 của luật quy định về tuyển dụng: bên tuyển dụng chỉ được yêu cầu thông tin phục vụ mục đích tuyển dụng, thông tin đó chỉ được dùng cho mục đích tuyển dụng hoặc mục đích khác theo thỏa thuận, và việc xử lý phải được người dự tuyển đồng ý. Một số bản tóm tắt của các hãng luật còn nêu nghĩa vụ xóa, hủy thông tin của người không được tuyển nếu không có thỏa thuận khác; điều kiện cụ thể phải đọc trong văn bản gốc.

Dùng công cụ AI bên ngoài để xử lý CV có nằm trong phạm vi ứng viên đã đồng ý hay không, và hồ sơ của người không trúng tuyển được giữ bao lâu, là câu hỏi cho người phụ trách pháp lý của công ty.

### Cách làm an toàn hơn

1. Chỉ dùng công cụ công ty đã cho phép xử lý dữ liệu ứng viên.
2. Trước khi dán, xóa họ tên, ảnh, ngày sinh, giới tính, địa chỉ, số điện thoại, email, tình trạng hôn nhân. Thay bằng mã (UV-017). Việc này giảm cả rủi ro dữ liệu lẫn thiên kiến.
3. Nhờ AI trích thông tin theo tiêu chí, không nhờ chấm điểm hay xếp hạng. Mời hay không mời do người đọc và quyết.

```text
Dưới đây là một CV đã ẩn thông tin cá nhân (mã UV-017) và 4 tiêu chí bắt buộc của vị trí.
Với từng tiêu chí, hãy trả về một dòng trong bảng:
- Tiêu chí
- Bằng chứng trong CV: trích nguyên văn câu liên quan, đặt trong ngoặc kép.
  Không có thì ghi "không thấy trong CV".
- Điều cần hỏi thêm khi phỏng vấn để làm rõ.

Không chấm điểm, không kết luận phù hợp hay không phù hợp, không nhận xét về cách trình bày
hay văn phong. Không suy đoán tuổi, giới tính hay hoàn cảnh cá nhân của ứng viên.

Tiêu chí: [dán]
CV: [dán]
```

"Không thấy trong CV" khác với "ứng viên không có": có người có đúng kinh nghiệm bạn cần nhưng viết bằng từ khác, và AI bỏ qua. Bảng này để đọc CV nhanh hơn và chuẩn bị câu hỏi, không phải bộ lọc.

**Cách kiểm tra:** tìm các câu trích trong CV gốc xem có thật không. Với mỗi đợt tuyển, lấy ngẫu nhiên khoảng mười CV mà bảng ghi nhiều "không thấy" rồi tự đọc đầy đủ. Nếu trong đó có người đáng mời, bạn biết cách trích đang bỏ sót và không nên dựa vào nó.

## Việc 4: Thư mời và thư từ chối tử tế

Thư từ chối hay bị bỏ lơ vì viết vừa ngại vừa tốn thời gian.

```text
Viết thư từ chối ứng viên sau vòng phỏng vấn, xưng "em" hoặc "chúng tôi" tùy bản, gọi "anh/chị".
Thông tin: vị trí [..], ứng viên đã phỏng vấn vòng [..] ngày [..].
Một điểm tôi ghi nhận thật ở ứng viên: [ví dụ: chuẩn bị kỹ về sản phẩm của công ty].
Yêu cầu:
- Nói kết quả ngay trong hai câu đầu, không vòng vo.
- Nhắc đúng một điểm ghi nhận ở trên, không thêm lời khen nào khác.
- Không nêu lý do từ chối cụ thể, không so sánh với ứng viên khác.
- Không hứa "sẽ liên hệ khi có vị trí phù hợp" trừ khi tôi ghi là có.
- Dưới 120 chữ. Viết 2 bản: thân mật và trang trọng.
```

Điểm ghi nhận phải do bạn đưa vào. Để AI tự khen, thư nào cũng "ấn tượng với kinh nghiệm phong phú của anh/chị" và ứng viên nhận ra ngay là thư mẫu.

Với thư mời nhận việc, AI chỉ lo phần lời văn. Mức lương, ngày bắt đầu, thời gian thử việc, phụ cấp phải chép từ quyết định đã được duyệt; dặn trong câu lệnh "chỉ dùng các thông tin sau, thiếu thì để [cần bổ sung]".

**Cách kiểm tra:** thư từ chối, đọc xem có câu nào nghe như lời hứa hoặc như lý do có thể bị hỏi vặn lại. Thư mời, so từng con số với bản đã duyệt và nhờ một người thứ hai đọc trước khi gửi.

## Việc 5: Tài liệu onboarding và FAQ nội quy

Nhân viên mới hỏi đi hỏi lại cùng những câu: xin nghỉ phép thế nào, đi muộn tính ra sao, đăng ký làm thêm giờ ở đâu. Câu trả lời đã có trong nội quy, chỉ là không ai đọc hết.

```text
Dưới đây là nội quy lao động và quy định nghỉ phép của công ty tôi.
Chỉ dựa vào văn bản này, hãy soạn 20 câu hỏi nhân viên mới hay hỏi nhất kèm câu trả lời.
Mỗi câu trả lời: tối đa 3 câu, giọng thân thiện, cuối câu ghi (Điều .., mục ..) nơi lấy thông tin.
Câu hỏi nào nhân viên hay hỏi mà văn bản không quy định, xếp riêng vào mục
"Chưa có quy định" để tôi hỏi lại ban giám đốc. Không tự suy ra câu trả lời từ luật
hay từ thông lệ ở công ty khác.

[Dán văn bản]
```

Mục "Chưa có quy định" cho bạn danh sách những chỗ nội quy còn trống, trước giờ xử lý bằng hỏi miệng.

**Cách kiểm tra:** mở đúng điều khoản được dẫn cho từng câu trả lời. Chỗ dễ sai nhất là con số (số ngày phép, thời hạn báo trước) và trường hợp ngoại lệ mà AI lược mất khi rút gọn. Nội quy đổi thì phải soạn lại FAQ, nên ghi ngày và phiên bản nội quy ở đầu tài liệu.

## Việc 6: Tổng hợp câu trả lời mở trong khảo sát nhân viên

Câu hỏi mở ("Điều gì bạn muốn công ty thay đổi nhất?") với 200 câu trả lời thường bị đọc lướt. AI nhóm chủ đề tốt, với hai lưu ý.

Thứ nhất, ẩn danh trước khi dán: xóa tên, email, phòng ban nếu phòng chỉ có vài người, và lược những chi tiết làm lộ người viết. Khảo sát đã hứa ẩn danh thì lời hứa đó phải còn nguyên khi dữ liệu rời khỏi Sheets.

Thứ hai, đừng tin AI đếm: hỏi "bao nhiêu người phàn nàn về lương" trên 200 dòng sẽ ra con số ước chừng. Tách thành hai bước:

```text
Bước 1. Dưới đây là 50 câu trả lời đầu tiên cho câu hỏi "[..]". Hãy đề xuất 6-10 nhãn chủ đề,
mỗi nhãn kèm định nghĩa một câu. Có nhãn "Khác".

Bước 2 (sau khi tôi duyệt bộ nhãn). Với từng câu trả lời tôi dán, trả về bảng 3 cột:
số thứ tự, nhãn chính, nhãn phụ (nếu có). Giữ nguyên số thứ tự tôi đánh, không gộp,
không bỏ dòng nào. Câu nào không rõ ý thì gắn "Khác".
```

Gắn nhãn theo lô khoảng 50 dòng, cùng một bộ nhãn đã duyệt cho mọi lô, rồi dán bảng nhãn trở lại Sheets và đếm bằng COUNTIF hoặc pivot table.

**Cách kiểm tra:** số dòng trả về bằng số dòng gửi đi. Tự gắn nhãn khoảng 20 câu ngẫu nhiên rồi so với AI. Lệch nhiều thì định nghĩa nhãn đang mơ hồ, sửa nhãn rồi chạy lại. Đọc hết nhóm "Khác", vì ý kiến hiếm nhưng nghiêm trọng (ví dụ phản ánh bị quấy rối) thường nằm ở đó và không được phép chìm trong thống kê.

## Việc nào giao, việc nào giữ

| Việc | Giao AI | Bạn giữ |
|---|---|---|
| JD | Hỏi lại cho đủ thông tin, viết nháp | Yêu cầu bắt buộc, lương và quyền lợi |
| Phỏng vấn | Câu hỏi, mô tả mức điểm, phiếu chấm | Hỏi, nghe, chấm, quyết định |
| CV | Trích bằng chứng theo tiêu chí trên bản đã ẩn danh | Đọc, chọn, loại |
| Thư từ | Lời văn, giọng điệu | Mọi con số, mọi lời hứa |
| Onboarding, FAQ | Soạn từ văn bản có sẵn, dẫn điều khoản | Đối chiếu, trả lời ca ngoại lệ |
| Khảo sát | Đề xuất nhãn, gắn nhãn | Đếm bằng công thức, đọc nhóm "Khác", kết luận |

Những việc không nên giao, ngoài chấm điểm và loại ứng viên đã nói ở việc 3:

- **Đánh giá hiệu suất, quyết định kỷ luật hay cho nghỉ việc.** Nhận định về một con người phải có người đứng tên và giải thích được. Bài [cho người quản lý nhóm](/bai-chia-se/ai-cho-quan-ly-hop-va-du-an/) có câu lệnh chuẩn bị buổi phản hồi mà không để AI nhận xét về nhân viên.
- **Trả lời câu hỏi pháp lý về lao động** như cách tính trợ cấp, thời gian thử việc, mức đóng bảo hiểm. Dữ liệu của AI có thể đã cũ; dùng nó để biết nên tra điều nào, rồi đọc văn bản gốc.
- **Xử lý hồ sơ nhân viên, bảng lương, hồ sơ sức khỏe** bằng công cụ chưa được công ty cho phép.

## Bắt đầu từ đâu

Nếu chọn một việc, mình đề nghị việc 2: bộ câu hỏi và phiếu chấm không đụng tới dữ liệu cá nhân của ai.

Mỗi vị trí tuyển lại dùng gần như cùng các câu lệnh trên với tiêu chí khác. Cách lưu chúng thành mẫu có trong bài [Từ hỏi từng câu đến một hệ thống làm việc với AI của riêng bạn](/bai-chia-se/xay-he-thong-ai-ca-nhan/).

## Nguồn đọc thêm

- [Doanh nghiệp khi chấm dứt hợp đồng phải xóa dữ liệu cá nhân người lao động (báo Tiền Phong)](https://tienphong.vn/doanh-nghiep-khi-cham-dut-hop-dong-phai-xoa-du-lieu-ca-nhan-nguoi-lao-dong-post1764533.tpo): bản tin tóm tắt Điều 25 về tuyển dụng và quản lý người lao động. Dùng để biết cần đọc điều nào trong văn bản gốc.
- Luật Bảo vệ dữ liệu cá nhân số 91/2025/QH15 và nghị định hướng dẫn thi hành: đọc toàn văn trên cổng thông tin pháp luật chính thức (bài không dẫn link vì chưa đối chiếu trực tiếp với toàn văn).
