# Chương trình Mentoring (gói 4 buổi)

Status: proposed (chờ người sáng lập duyệt; chạy thử với 5 suất beta rồi chỉnh)

Chi tiết hóa mục Mentoring trong `services.md`. Các ràng buộc ở đó giữ nguyên: 4 buổi × 60 phút trong
1 tháng, kết quả là một tool/automation/skill AI chạy được, vào gói qua Tư vấn qua chat, tối đa 1–2 học viên
cùng lúc. Nội dung hiển thị trên trang Mentoring lấy từ `src/mentoring-program.ts`; sửa chương trình thì sửa
cả hai chỗ.

## Nguyên tắc thiết kế

1. **Một việc, làm tới nơi.** Cả gói chỉ giải một việc lặp lại của học viên. Không dạy AI tổng quát: phần đó
   đã có Bài chia sẻ.
2. **Việc thật, dữ liệu thật (đã ẩn danh).** Mọi buổi làm trên đầu vào thật của học viên, không dùng ví dụ giả.
3. **Bộ mẫu kiểm tra đi trước.** Trước khi dựng, gom 5–10 đầu vào thật kèm kết quả mong muốn. Mọi lần sửa đều
   chạy lại trên bộ mẫu để biết tốt lên hay tệ đi. Đây là thói quen eval, thu nhỏ cho người không làm kỹ thuật.
4. **Dùng thật giữa các buổi.** Buổi sau sửa theo nhật ký sử dụng của tuần trước, không theo cảm giác.
5. **Tự chủ là đích.** Buổi 4 học viên tự sửa, mentor chỉ quan sát. Gói thành công khi học viên không cần
   mentor để dùng và chỉnh thứ đã build.

## Trước gói: chốt đề bài (trong Tư vấn qua chat, miễn phí)

Học viên trả lời **phiếu khởi động** qua chat:

1. Bạn làm công việc gì, hằng tuần lặp lại những việc nào?
2. Việc nào bạn muốn AI làm giúp nhất? Hiện mỗi lần mất bao lâu, mỗi tuần mấy lần?
3. Đầu vào của việc đó là gì (file, email, tin nhắn, form…) và đầu ra phải trông thế nào?
4. Làm sao biết kết quả đúng? Ai kiểm tra?
5. Dữ liệu có gì nhạy cảm không? Công ty cho phép dùng công cụ AI nào?
6. Bạn đang dùng công cụ AI nào, có tài khoản trả phí không?

Mentor chấm việc được chọn theo 4 tiêu chí, mỗi tiêu chí đạt/không:

| Tiêu chí | Câu hỏi kiểm tra |
|---|---|
| Lặp lại | Có làm ít nhất mỗi tuần một lần không? |
| Đầu vào rõ | Đầu vào có dạng ổn định, lấy được mẫu thật không? |
| Kiểm tra được | Học viên có tự nhìn ra kết quả đúng hay sai trong vài phút không? |
| Sai không gây hại lớn | Nếu AI sai mà lọt, có sửa lại được không? Có người duyệt trước khi ra ngoài không? |

Đủ 4/4 mới nhận. Thiếu thì cùng học viên thu hẹp việc hoặc chọn việc khác; không chọn được thì nói thẳng là
chưa hợp Mentoring và gợi ý Bài chia sẻ phù hợp.

## Ba hướng

Hướng chọn theo việc, không theo sở thích công cụ. Mặc định là hướng đơn giản nhất giải được việc.

| Hướng | Là gì | Hợp khi | Học viên cần | Ví dụ |
|---|---|---|---|---|
| **Skill** | Một trợ lý chuyên việc: hướng dẫn cố định + tài liệu mẫu + câu lệnh, chạy trong không gian dự án của công cụ chat | Việc soạn, tóm tắt, rà soát, trả lời theo mẫu | Một tài khoản công cụ chat AI | Trợ lý viết JD theo mẫu công ty; trợ lý trả lời khách theo chính sách |
| **Automation** | Một luồng tự chạy nối các ứng dụng: có đầu vào mới thì AI xử lý rồi ghi/gửi kết quả | Việc lặp theo lịch hoặc theo sự kiện, đầu vào đến từ form, email, bảng tính | Tài khoản công cụ tự động hóa (mentor gợi ý theo nhu cầu) | Phản hồi khách từ form → phân loại → ghi bảng → báo nhóm |
| **Tool** | Một chương trình nhỏ chạy trên máy học viên, mentor viết cùng với coding agent | Xử lý file, đối chiếu dữ liệu, việc cần tính chính xác | Máy tính cài được phần mềm; mentor lo phần code | Đối chiếu sao kê với sổ; gộp và soát nhiều file Excel |

## Bốn buổi

Mỗi buổi 60 phút, gọi online, chia sẻ màn hình. Mỗi buổi kết thúc bằng một sản phẩm cụ thể và một bài tập.

### Buổi 1 (tuần 1): Khảo sát và chốt đề bài

Mục tiêu: hai bên hiểu giống nhau về việc cần làm và thế nào là "xong".

| Phút | Hoạt động |
|---|---|
| 0–15 | Học viên chia sẻ màn hình, làm việc đó một lượt như thường ngày. Mentor chỉ quan sát và hỏi. |
| 15–30 | Cùng vẽ lại quy trình thành các bước; đánh dấu bước nào tốn thời gian, bước nào cần phán đoán của người. |
| 30–50 | Viết **đề bài một trang**: đầu vào, đầu ra, tiêu chí đạt, việc AI không được làm, ranh giới dữ liệu. Chọn hướng. |
| 50–60 | Dựng bản thô đầu tiên ngay trong buổi (một câu lệnh, một bước) để học viên thấy hướng đi. |

Sản phẩm: đề bài một trang; số đo gốc (việc này đang tốn bao nhiêu phút mỗi lần).
Bài tập: gom 5–10 đầu vào thật kèm kết quả mong muốn (bộ mẫu kiểm tra), ẩn danh nếu cần; bấm giờ 2 lần làm tay.

### Buổi 2 (tuần 2): Dựng bản chạy được

Mục tiêu: học viên tự chạy được phiên bản 1 trên tài khoản/máy của mình.

| Phút | Hoạt động |
|---|---|
| 0–25 | Chạy bản thô trên bộ mẫu. Đọc từng kết quả, ghi lỗi và gom lỗi thành nhóm. |
| 25–50 | Sửa theo nhóm lỗi lớn nhất: viết lại hướng dẫn, thêm tài liệu mẫu, tách bước, hoặc nối luồng/viết code tùy hướng. |
| 50–60 | Học viên tự chạy lại từ đầu, không có mentor cầm tay. Chốt cách dùng hằng ngày. |

Sản phẩm: phiên bản 1 chạy được; bảng nhóm lỗi.
Bài tập: dùng phiên bản 1 cho việc thật ít nhất 3 lần, ghi **nhật ký sử dụng** (ngày, đầu vào, đạt/chưa, phải sửa tay
gì, mất bao lâu).

### Buổi 3 (tuần 3): Sửa theo việc thật

Mục tiêu: xử lý những gì chỉ lộ ra khi dùng thật.

| Phút | Hoạt động |
|---|---|
| 0–15 | Soát nhật ký sử dụng; chọn 2–3 vấn đề đáng sửa nhất. |
| 15–45 | Sửa, chạy lại trên bộ mẫu để chắc không làm hỏng cái đang đúng. Bổ sung trường hợp mới vào bộ mẫu. |
| 45–60 | Thêm bước tự kiểm tra và cách xử lý khi đầu vào lạ; viết danh sách kiểm tra trước khi dùng kết quả. |

Sản phẩm: phiên bản 2; danh sách kiểm tra trước khi dùng kết quả.
Bài tập: dùng phiên bản 2 cả tuần, cố không hỏi mentor; ghi lại chỗ kẹt và chỗ muốn đổi.

### Buổi 4 (tuần 4): Bàn giao và tự chủ

Mục tiêu: học viên tự bảo trì được, và biết làm việc tiếp theo một mình.

| Phút | Hoạt động |
|---|---|
| 0–20 | Học viên tự làm một thay đổi nhỏ từ danh sách "muốn đổi"; mentor quan sát, chỉ nhắc khi kẹt. |
| 20–35 | Viết trang hướng dẫn sử dụng và bảo trì: dùng thế nào, khi sai thì xem ở đâu, khi nào cần sửa. |
| 35–45 | Đo lại thời gian, so với số đo gốc. Ghi nhận cả phần chưa đạt. |
| 45–60 | Chọn việc tiếp theo trong danh sách công việc của học viên và phác kế hoạch tự làm. Trao đổi về case study. |

Sản phẩm: phiên bản cuối; trang hướng dẫn; số đo trước/sau; tư liệu cho case study.

## Giữa các buổi

- Học viên nhắn qua kênh chat khi kẹt; mentor trả lời bất đồng bộ.
- Hướng Tool: phần code không kịp trong buổi, mentor hoàn thiện giữa hai buổi và giải thích lại ở đầu buổi sau.
  Học viên luôn giữ toàn bộ mã nguồn và tài liệu.

## Tiêu chí hoàn thành gói

- Học viên tự chạy được sản phẩm mà không cần mentor.
- Đã dùng cho việc thật ít nhất 5 lần.
- Có số đo trước/sau, kể cả khi kết quả không như kỳ vọng.
- Có trang hướng dẫn sử dụng và bảo trì.

Nếu hết 4 buổi chưa đạt: nói rõ còn thiếu gì; học viên có thể mua buổi lẻ để làm tiếp.

## Ai hợp, ai chưa hợp

Hợp: có một việc lặp lại cụ thể; sẵn sàng dùng thật giữa các buổi (khoảng 1–2 giờ mỗi tuần); được phép dùng
công cụ AI với dữ liệu đó (hoặc ẩn danh được).

Chưa hợp: muốn học AI tổng quát (đọc Bài chia sẻ); việc chỉ làm một lần; dữ liệu bị cấm đưa ra ngoài và không
ẩn danh được; cần một phần mềm lớn nhiều người dùng (ngoài phạm vi 4 buổi).

## Câu hỏi mở cho người sáng lập

- Thời gian mentor làm giữa các buổi (hướng Tool) có cần giới hạn rõ số giờ không?
- Chi phí công cụ (tài khoản trả phí, công cụ tự động hóa) do học viên trả: có cần ghi rõ trên trang không?
  Hiện trang đã ghi.
- Có nhận hướng Tool trong 5 suất beta không, hay ưu tiên Skill/Automation để dễ bàn giao hơn?
