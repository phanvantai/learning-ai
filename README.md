# Learning AI

Website chia sẻ, mentoring và đào tạo ứng dụng AI cho công việc hằng ngày.

- Spec sản phẩm: `docs/specs/`
- Thuật ngữ: `CONTEXT.md`

## Phát triển

Cần Node.js 24 (xem `.node-version`; Astro yêu cầu tối thiểu 22.23.4).

```sh
npm install
npm run dev      # chạy local
npm test         # chạy test
npm run check    # kiểm tra kiểu
npm run build    # build ra dist/
```

Deploy: Vercel tự build khi merge vào `main`; mỗi PR có một bản preview.

Chỗ còn `TODO(owner)` là nội dung thật người sáng lập cần tự điền (`src/site.ts`, trang Giới thiệu, `src/contact-form.ts`).

### Bật form liên hệ trang Doanh nghiệp (`TODO(owner)`)

Web tĩnh không có backend, nên form liên hệ trên `/doanh-nghiep/` gửi qua [Formspree](https://formspree.io) (gói miễn phí đủ dùng cho giai đoạn thí điểm). Khi chưa cấu hình, form bị khóa và hướng khách sang kênh chat, không ai gửi vào hư không.

1. Đăng ký Formspree bằng email muốn nhận liên hệ (email người sáng lập) và xác nhận email.
2. Tạo form mới (tên gợi ý: "Learning AI – Doanh nghiệp"), copy địa chỉ dạng `https://formspree.io/f/abcdwxyz`.
3. Dán địa chỉ đó vào `BUSINESS_CONTACT_FORM_ENDPOINT` trong `src/contact-form.ts` (chỗ duy nhất cần sửa), mở PR rồi merge.
4. Trên bản preview của PR, gửi thử một lần và kiểm tra hộp thư (cả thư mục spam).

Có JavaScript thì khách thấy thông báo thành công/lỗi bằng tiếng Việt ngay trên trang; tắt JavaScript thì form vẫn gửi được, nhưng Formspree hiện trang cảm ơn mặc định của họ.
