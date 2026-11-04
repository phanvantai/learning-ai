---
title: Dùng AI tóm tắt email dài trong vài phút (và chỗ mình từng làm sai)
date: 2026-11-04
tags: [Email, Tóm tắt, Prompt mẫu]
description: Một câu lệnh mẫu để AI tóm tắt chuỗi email dài thành việc cần làm, hạn chót và người phụ trách, kèm những lỗi mình đã gặp khi thử.
---

Mỗi sáng thứ Hai, hộp thư của mình thường có vài chuỗi email 20–30 thư trả lời qua lại: người thì "đồng ý", người thì đổi ý, người thì chỉ thêm một dòng ở cuối. Đọc hết mất cả buổi sáng. Mấy tháng nay mình nhờ AI (ChatGPT, Gemini hoặc Claude đều được) đọc trước và tóm tắt. Bài này chia sẻ cách mình đang làm, cả chỗ chưa ổn.

## Cách làm: 3 bước

1. **Sao chép cả chuỗi email** (từ thư cũ nhất đến mới nhất) vào khung chat của công cụ AI bạn đang dùng.
2. **Dán câu lệnh mẫu** bên dưới vào phía trên đoạn email.
3. **Đọc bản tóm tắt, rồi mở lại email gốc** để kiểm tra những chỗ quan trọng: con số, ngày tháng, ai đồng ý cái gì.

## Câu lệnh mẫu

```text
Bạn là trợ lý đọc email giúp tôi. Dưới đây là một chuỗi email, sắp xếp từ cũ đến mới.
Hãy tóm tắt bằng tiếng Việt theo đúng 4 mục:

1. Chuyện gì đang diễn ra (tối đa 3 câu).
2. Những gì đã được chốt, ghi rõ ai chốt.
3. Việc cần làm: mỗi dòng gồm việc gì, ai làm, hạn chót (nếu email không nói thì ghi "chưa rõ").
4. Những câu hỏi còn bỏ ngỏ mà tôi cần trả lời.

Chỉ dùng thông tin có trong email. Chỗ nào không chắc thì ghi "không chắc" thay vì đoán.

[Dán chuỗi email vào đây]
```

Hai câu cuối của câu lệnh là quan trọng nhất. Lúc đầu mình không có chúng, và AI rất hay "tự điền" hạn chót cho có vẻ đầy đủ.

## Những chỗ mình từng làm sai

- **Tin bản tóm tắt 100%.** Có lần AI ghi "anh Minh đã đồng ý" trong khi thư cuối anh ấy chỉ nói "để anh xem lại". Từ đó, phần "đã được chốt" mình luôn mở email gốc ra đối chiếu.
- **Dán email theo thứ tự lộn xộn.** Phần lớn ứng dụng email hiện thư mới nhất ở trên cùng. Nếu không nói rõ thứ tự, AI có thể hiểu nhầm ý kiến cũ là ý kiến mới.
- **Dán cả thông tin nhạy cảm.** Hợp đồng, lương thưởng, số tài khoản, dữ liệu khách hàng: đừng dán vào công cụ AI miễn phí. Hãy hỏi công ty bạn đang cho phép dùng công cụ nào, hoặc xóa các chi tiết đó trước khi dán.

## Kết quả thực tế

Với một chuỗi khoảng 25 thư, mình mất chừng 5 phút để dán, đọc tóm tắt và mở lại vài thư quan trọng để đối chiếu, thay vì 15–20 phút đọc từng thư. Bản tóm tắt không hoàn hảo: thỉnh thoảng nó bỏ sót một câu hỏi nằm lẫn giữa thư. Nhưng nó cho mình một "bản đồ" để biết cần đọc kỹ thư nào.

Nếu bạn thử và có cách viết câu lệnh hay hơn, nhắn cho mình nhé. Mình cũng đang học mỗi tuần.
