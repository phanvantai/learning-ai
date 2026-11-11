---
title: "Từ hỏi từng câu đến một hệ thống làm việc với AI của riêng bạn"
date: 2026-11-11
tags: [Tư duy dùng AI, Quy trình làm việc]
description: Cách kiểm kê công việc một tuần, chấm điểm việc nào đáng giao AI, rồi dựng hồ sơ ngữ cảnh, không gian theo dự án và thư viện câu lệnh, kèm nhịp duy trì 15 phút mỗi tuần và cách đo xem có tiết kiệm thật không.
loai: ung-dung
---

Bài [Cùng một AI, vì sao người khác dùng hiệu quả hơn bạn?](/bai-chia-se/cung-ai-nguoi-dung-hieu-qua-hon/) nói về năm thói quen khi làm việc với AI. Thói quen thì dễ rơi rụng: tuần bận là quên, và bạn quay về mở khung chat, gõ một câu, nhận một câu.

Bài này đi tiếp một bước: biến các thói quen đó thành một hệ thống có sẵn, để lần sau gặp việc quen bạn không phải nghĩ lại từ đầu. Hệ thống ở đây không cần lập trình. Nó gồm bốn thứ: một danh sách việc đã chọn lọc, một hồ sơ ngữ cảnh, vài không gian làm việc theo dự án, và một thư viện câu lệnh. Dựng lần đầu mất khoảng hai đến ba giờ, rải trong một tuần.

```text
Kiểm kê việc 1 tuần
      |
      v
Chấm điểm, chọn 3-5 việc đáng giao
      |
      v
Hồ sơ ngữ cảnh  ->  Không gian theo dự án  ->  Thư viện câu lệnh
      ^                                              |
      |                                              v
      +-------  Rà soát 15 phút mỗi tuần  <----  Dùng và đo
```

## Bước 1: Kiểm kê công việc một tuần

Hầu hết mọi người chọn việc giao AI theo cảm hứng: thấy ai đó khoe một câu lệnh hay thì thử. Cách chắc hơn là nhìn vào tuần làm việc thật của chính mình.

Trong năm ngày làm việc, mỗi khi xong một việc mất hơn 15 phút, ghi một dòng vào file ghi chú: việc gì, mất bao lâu, đầu vào là gì, đầu ra là gì. Không cần chính xác tới phút. Cuối tuần bạn sẽ có khoảng 20 đến 40 dòng, trong đó nhiều dòng lặp lại. Những dòng lặp lại là thứ cần nhìn kỹ.

Nếu ngại ghi, cuối mỗi ngày mở lịch, hộp thư đã gửi và lịch sử tin nhắn công việc ra rồi chép lại. Trí nhớ cuối tuần thường bỏ sót chính những việc vặt lặp đi lặp lại.

## Bước 2: Chấm điểm việc nào đáng giao

Với mỗi loại việc, chấm bốn tiêu chí, mỗi tiêu chí từ 0 đến 2 điểm:

| Tiêu chí | 0 điểm | 1 điểm | 2 điểm |
|---|---|---|---|
| Lặp lại | Vài tháng một lần | Hằng tháng | Hằng tuần trở lên |
| Đầu vào rõ | Nằm trong đầu bạn hoặc phải đi hỏi nhiều người | Có nhưng rải rác | Là văn bản, bảng, file có sẵn |
| Kiểm tra được | Phải làm lại từ đầu mới biết đúng sai | Đọc kỹ thì biết | Có cách đối chiếu nhanh (tổng khớp, so với bản gốc) |
| Sai không gây hại lớn | Sai là mất tiền, mất uy tín, vướng pháp lý | Sai thì phiền nhưng sửa được | Sai thì sửa trong vài phút, chưa ai thấy |

Bảng chấm mẫu cho một nhân viên văn phòng (ví dụ minh họa, không phải số liệu khảo sát):

| Việc | Lặp lại | Đầu vào | Kiểm tra | Hại khi sai | Tổng | Quyết định |
|---|---|---|---|---|---|---|
| Tóm tắt chuỗi email dài | 2 | 2 | 2 | 2 | 8 | Giao AI, làm mẫu câu lệnh |
| Soạn nháp email trả lời khách | 2 | 2 | 1 | 1 | 6 | Giao AI soạn, bạn duyệt từng thư |
| Viết báo cáo tuần | 2 | 1 | 1 | 1 | 5 | Giao AI sau khi gom đầu vào về một chỗ |
| Làm slide cho họp quý | 0 | 1 | 1 | 1 | 3 | Dùng AI khi cần, chưa đáng dựng mẫu |
| Đối chiếu công nợ | 2 | 2 | 1 | 0 | 5 | Xem ghi chú bên dưới |
| Đàm phán giá với nhà cung cấp | 1 | 0 | 0 | 0 | 1 | Giữ lại |

Cách đọc bảng:

- **Từ 6 điểm trở lên:** đáng đầu tư dựng mẫu câu lệnh và không gian riêng.
- **3 đến 5 điểm:** dùng AI cho từng phần, hoặc sửa điểm yếu trước. Ví dụ báo cáo tuần chỉ được 1 điểm đầu vào vì dữ liệu rải rác. Gom nó về một chỗ là điểm tăng lên.
- **Dưới 3 điểm:** giữ lại.

Có một ngoại lệ quan trọng: cột "hại khi sai" có quyền phủ quyết. Việc được 0 điểm ở cột này (như đối chiếu công nợ trong bảng) thì dù tổng điểm cao cũng chỉ nên giao AI phần chuẩn bị, còn con số cuối cùng bạn tự kiểm từng dòng. Lý do là AI tạo sinh có thể đưa ra kết quả sai với giọng rất tự tin, và việc có hại lớn không chịu được kiểu sai đó.

Chọn ba đến năm việc điểm cao nhất để bắt đầu. Dựng hệ thống cho hai mươi việc cùng lúc là cách nhanh nhất để bỏ cuộc.

## Bước 3: Viết hồ sơ ngữ cảnh cá nhân

Bài trước có một mẫu hướng dẫn cố định bốn dòng. Hồ sơ ngữ cảnh là bản đầy đủ hơn của ý đó: một tài liệu khoảng nửa trang đến một trang, trả lời trước những câu mà một đồng nghiệp mới sẽ phải hỏi bạn. Hầu hết công cụ có chỗ lưu loại hướng dẫn này để áp dụng cho mọi cuộc trò chuyện.

```text
VAI TRÒ
Tôi là [vị trí] tại [loại công ty, quy mô, ngành]. Tôi chịu trách nhiệm về [2-3 mảng].
Người tôi báo cáo: [vai trò]. Người tôi hay làm việc cùng: [vai trò].

NGƯỜI ĐỌC THƯỜNG GẶP
- Sếp trực tiếp: thích ngắn, kết luận trước, số liệu sau.
- Khách hàng: [đặc điểm], xưng hô [anh/chị, em].
- Nội bộ nhóm: thân mật, dùng được từ viết tắt.

TỪ NGỮ RIÊNG
[Từ viết tắt, tên sản phẩm, tên quy trình nội bộ và nghĩa của chúng]

CÁCH TRẢ LỜI
Tiếng Việt. Đi thẳng vào việc, không mở bài, không tóm tắt lại ở cuối.
Dùng bảng khi so sánh từ 3 mục trở lên.
Thiếu thông tin thì hỏi lại tối đa 3 câu trước khi làm.

GIỚI HẠN
Không bịa số liệu, tên người, ngày tháng, điều khoản. Không chắc thì ghi "cần kiểm tra".
Khi tôi dán tài liệu, chỉ dùng thông tin trong tài liệu trừ khi tôi nói khác.
```

Ba lưu ý khi viết:

- **Không đưa bí mật vào hồ sơ.** Hồ sơ được gửi kèm mọi cuộc trò chuyện. Viết "công ty phân phối thiết bị y tế, khoảng 50 người" là đủ, không cần tên khách hàng hay doanh số.
- **Mục "từ ngữ riêng" có tác dụng lớn hơn vẻ ngoài.** AI không biết "BBNT" ở công ty bạn là biên bản nghiệm thu. Thiếu mục này, nó đoán, và đôi khi đoán sai một cách trôi chảy.
- **Giữ ngắn.** Hồ sơ dài ba trang thì các chỉ dẫn bắt đầu mâu thuẫn nhau và bạn không còn biết dòng nào đang có tác dụng. Mỗi dòng nên có lý do tồn tại: nó sửa một lỗi bạn từng gặp.

## Bước 4: Tổ chức không gian theo dự án

Hồ sơ ngữ cảnh nói về bạn. Mỗi loại việc lại cần thêm bối cảnh riêng: tài liệu của dự án, mẫu báo cáo, giọng văn với một khách hàng cụ thể. Các công cụ lớn đều có một dạng không gian riêng để chứa thứ này, gồm hai phần chung: một bộ hướng dẫn riêng và một nhóm tài liệu tham chiếu. Tên gọi thì khác nhau và thay đổi theo thời gian: Projects ở ChatGPT và Claude, Gems ở Gemini, GPTs ở ChatGPT.

Vì tên gọi và giới hạn theo gói thay đổi liên tục, đừng học thuộc menu. Ví dụ, theo trang trợ giúp của Google mà mình đọc lúc viết bài này, từ tháng 11/2026 Gems trên tài khoản Google cá nhân sẽ chuyển thành "skills". Điều cần nhớ là khả năng: một chỗ lưu hướng dẫn và tài liệu để mọi cuộc trò chuyện trong đó đều có sẵn bối cảnh. Công cụ bạn dùng gọi nó là gì thì xem trang trợ giúp hiện tại của công cụ đó.

Nguyên tắc chia: mỗi không gian cho một loại việc lặp lại hoặc một dự án đang chạy, lấy từ danh sách điểm cao ở bước 2. Ví dụ:

| Không gian | Hướng dẫn riêng | Tài liệu đính kèm |
|---|---|---|
| Báo cáo tuần | Khung báo cáo, người nhận, giọng văn | Hai báo cáo cũ được khen, mẫu của công ty |
| Email khách hàng | Cách xưng hô, những điều không được hứa | Bảng chính sách đổi trả, câu trả lời mẫu |
| Dự án A | Mục tiêu, mốc thời gian, ai phụ trách gì | Kế hoạch, biên bản họp gần nhất |

Hai lỗi hay gặp ở bước này. Một là nhồi mọi tài liệu vào một không gian: tài liệu càng nhiều và càng lạc đề thì câu trả lời càng loãng, nên chỉ để những gì việc đó thật sự cần. Hai là để tài liệu cũ nằm lại: biên bản họp từ ba tháng trước có thể khiến AI trả lời theo quyết định đã bị thay. Trước khi tải tài liệu công ty lên, kiểm tra xem công ty cho phép dùng công cụ nào với loại dữ liệu đó.

## Bước 5: Thư viện câu lệnh cá nhân

Bài trước khuyên ghi lại câu lệnh hiệu quả vào sổ. Thư viện là cuốn sổ đó khi được tổ chức để tìm lại trong mười giây và ngày càng tốt lên.

**Nơi lưu:** một file duy nhất (Google Docs, Notion, ghi chú trên máy), mỗi câu lệnh một mục. Ưu tiên chỗ bạn mở được cả trên máy tính lẫn điện thoại.

**Cách đặt tên:** động từ, đối tượng, biến thể, phiên bản. Ví dụ:

```text
tom-tat_chuoi-email_v3
soan_email-tu-choi_khach-hang_v2
viet_bao-cao-tuan_gui-sep_v4
kiem-tra_hop-dong_dieu-khoan-thanh-toan_v1
```

Bắt đầu bằng động từ giúp các câu lệnh cùng loại việc nằm cạnh nhau khi sắp xếp. Số phiên bản giúp bạn quay lại bản trước nếu bản sửa hóa ra tệ hơn.

**Mỗi mục gồm bốn phần:**

```text
TÊN: viet_bao-cao-tuan_gui-sep_v4
DÙNG KHI: chiều thứ Sáu, đã có bảng công việc và tin nhắn nhóm trong tuần
CÂU LỆNH: [nội dung, phần cần thay đặt trong ngoặc vuông]
GHI CHÚ: v4 thêm yêu cầu ghi nguồn cho từng ý vì v3 hay tự suy diễn tiến độ.
         Vẫn còn lỗi: đôi khi gộp hai việc khác nhau thành một dòng.
```

Phần ghi chú là thứ biến một bộ sưu tầm thành thư viện. Quy tắc cải tiến: mỗi lần kết quả sai theo cùng một kiểu đến lần thứ hai, sửa câu lệnh gốc trong thư viện chứ đừng chỉ sửa trong cuộc trò chuyện. Sửa trong cuộc trò chuyện thì lần sau lỗi quay lại.

Câu lệnh sưu tầm trên mạng chỉ nên vào thư viện sau khi bạn đã dùng thật ít nhất hai lần và sửa cho hợp việc của mình. Một thư viện mười câu lệnh đã dùng mòn có giá trị hơn hai trăm câu chưa thử.

## Bước 6: Chọn đúng chế độ cho từng việc

Các công cụ AI hiện có nhiều chế độ làm việc, và chọn sai chế độ thì tốn thời gian chờ hoặc nhận kết quả hời hợt.

| Tình huống | Nên dùng | Vì sao |
|---|---|---|
| Soạn, sửa, tóm tắt, sắp ý trên tài liệu bạn đã có | Chat thường | Nhanh, sửa qua lại được, bạn kiểm soát đầu vào |
| Cần tìm hiểu một chủ đề từ nhiều nguồn trên mạng, chấp nhận chờ | Chế độ nghiên cứu sâu (deep research) | Nó tự tìm nhiều lượt, đọc nhiều nguồn rồi viết báo cáo có dẫn nguồn |
| Cùng một việc chạy lặp lại theo lịch hoặc theo sự kiện, đầu vào và đầu ra cố định | Công cụ tự động hóa | Không cần bạn ngồi dán từng lần |

Vài điều cần biết về chế độ nghiên cứu sâu. Nó chậm hơn nhiều so với chat: trang trợ giúp của Gemini ghi báo cáo thường mất khoảng 5 đến 10 phút, việc phức tạp thì lâu hơn. Nó thường bị giới hạn số lượt, hoặc chỉ có ở gói trả phí tùy công cụ. Và có dẫn nguồn không có nghĩa là đúng: hãy mở nguồn của những ý bạn định dùng, vì nó vẫn có thể hiểu sai nguồn hoặc lấy từ trang kém tin cậy. Với câu hỏi mà bạn đã có sẵn tài liệu trong tay, chat thường kèm tài liệu cho kết quả nhanh và dễ kiểm tra hơn.

Về tự động hóa (các công cụ nối ứng dụng với nhau như Zapier, Make, n8n, hoặc tính năng tác vụ theo lịch có sẵn trong vài công cụ AI): chỉ nghĩ tới khi một việc đã chạy ổn bằng tay với cùng một câu lệnh ít nhất vài tuần, và đạt 2 điểm ở cả cột kiểm tra lẫn cột hại khi sai. Tự động hóa một quy trình chưa ổn thì bạn chỉ sản xuất lỗi nhanh hơn, và không còn ai nhìn từng kết quả để phát hiện. Với việc gửi ra ngoài (email cho khách, tin nhắn cho đối tác), giữ một bước bạn duyệt trước khi gửi.

## Bước 7: Nhịp duy trì 15 phút mỗi tuần

Hệ thống không được bảo trì sẽ cũ đi: tài liệu hết hạn, câu lệnh không còn hợp, hồ sơ mô tả công việc của bạn nửa năm trước. Đặt một lịch cố định 15 phút, ví dụ chiều thứ Sáu, với năm việc:

1. **Nhìn lại tuần (3 phút):** việc nào AI làm tốt, việc nào phải sửa nhiều?
2. **Sửa một câu lệnh (5 phút):** chọn câu lệnh gây phiền nhất tuần, sửa bản gốc, tăng số phiên bản, ghi chú lý do.
3. **Dọn không gian (3 phút):** gỡ tài liệu đã cũ, thêm tài liệu mới của dự án đang chạy.
4. **Cập nhật hồ sơ (2 phút):** có lỗi nào lặp lại đáng thêm một dòng chỉ dẫn không? Có dòng nào thừa cần bỏ không?
5. **Ghi một con số (2 phút):** xem phần đo bên dưới.

Mỗi tuần chỉ sửa một thứ. Sửa nhiều thứ cùng lúc thì bạn không biết thay đổi nào tạo ra khác biệt.

## Đo xem có tiết kiệm thật không

Cảm giác "nhanh hơn" khi dùng AI không đáng tin lắm: thời gian chờ và thời gian sửa thường không được tính vào. Cách đo đơn giản, đủ dùng cho cá nhân:

**Lấy mốc trước.** Bảng kiểm kê ở bước 1 đã có thời gian làm tay cho từng việc. Đó là mốc so sánh.

**Tính đủ ba khoản khi làm với AI:** thời gian chuẩn bị đầu vào và viết yêu cầu, thời gian kiểm tra, thời gian sửa. Ghi tổng lại sau mỗi lần làm, trong khoảng bốn tuần.

```text
Việc: báo cáo tuần
Làm tay (mốc):        60 phút
Với AI, tuần 1:       20 chuẩn bị + 15 kiểm tra + 25 sửa = 60 phút
Với AI, tuần 4:       10 chuẩn bị + 10 kiểm tra +  5 sửa = 25 phút
```

Các con số trên là ví dụ minh họa. Điểm cần chú ý là hình dạng: tuần đầu thường không tiết kiệm gì, thậm chí chậm hơn, vì bạn còn đang sửa câu lệnh và gom đầu vào. Nếu sau bốn tuần tổng thời gian vẫn không giảm rõ, có ba khả năng: câu lệnh chưa đủ tốt, đầu vào vẫn lộn xộn, hoặc việc này không hợp giao AI. Trường hợp cuối thì gạch nó khỏi danh sách, và đó cũng là một kết quả có ích.

**Theo dõi thêm hai dấu hiệu chất lượng:** số lỗi lọt qua tới người nhận (sếp hoặc khách phát hiện trước bạn), và số lần bạn bỏ bản AI viết để làm lại từ đầu. Tiết kiệm 30 phút mà để lọt một con số sai tới khách hàng thì không phải tiết kiệm.

Cũng nên thành thật về chi phí: thời gian dựng ban đầu, 15 phút mỗi tuần, và tiền gói trả phí nếu có. Với việc chỉ làm mỗi quý một lần, hệ thống này có thể không bao giờ hoàn vốn. Đó là lý do bước chấm điểm đặt "lặp lại" lên đầu.

## Bắt đầu từ đâu nếu chỉ có một giờ

Làm theo thứ tự này và dừng ở đâu cũng được:

1. Liệt kê theo trí nhớ mười việc lặp lại trong tuần, chấm điểm, chọn một việc cao điểm nhất (20 phút).
2. Viết hồ sơ ngữ cảnh bản đầu, lưu vào công cụ bạn đang dùng (15 phút).
3. Viết câu lệnh cho việc đã chọn, chạy thử, sửa một lần, lưu vào file thư viện với tên và ghi chú (25 phút).

Tuần sau thêm việc thứ hai. Không gian theo dự án, chế độ nghiên cứu sâu và tự động hóa có thể đợi tới khi ba bước này đã thành nếp.

Nếu bạn muốn có người cùng làm thay vì tự dựng một mình, trang [Mentoring](/mentoring/) có lộ trình 4 buổi.

## Nguồn đọc thêm

- [What are projects?](https://support.claude.com/en/articles/9517075-what-are-projects), trang trợ giúp của Claude. Mô tả rõ hai thành phần của một không gian theo dự án: hướng dẫn riêng và tài liệu tham chiếu.
- [Trang trợ giúp về Gems của Gemini](https://support.google.com/gemini/answer/15146780). Cách tạo một trợ lý có hướng dẫn và tài liệu riêng, kèm thông báo chuyển Gems sang skills trên tài khoản cá nhân.
- [Trang trợ giúp về Deep Research của Gemini](https://support.google.com/gemini/answer/15719111). Giải thích cách chế độ nghiên cứu sâu lập kế hoạch, đọc nguồn và viết báo cáo, cùng thời gian chờ và giới hạn.
- [Using Research on Claude](https://support.claude.com/en/articles/11088861-using-research-on-claude), trang trợ giúp của Claude. Đọc để thấy chế độ nghiên cứu khác chat thường ở đâu và các giới hạn đi kèm.
