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

Web tĩnh không có backend, nên form liên hệ trên `/doanh-nghiep/` gửi qua [Formspree](https://formspree.io) (gói miễn phí có giới hạn số lần gửi mỗi tháng, đủ cho giai đoạn thí điểm; xem hạn mức hiện tại trên trang giá của Formspree). Khi chưa cấu hình, form bị khóa và hướng khách sang kênh chat, không ai gửi vào hư không.

1. Đăng ký Formspree bằng email muốn nhận liên hệ (email người sáng lập) và xác nhận email.
2. Tạo form mới (tên gợi ý: "Learning AI – Doanh nghiệp"), copy địa chỉ dạng `https://formspree.io/f/abcdwxyz`.
3. Dán địa chỉ đó vào `BUSINESS_CONTACT_FORM_ENDPOINT` trong `src/contact-form.ts` (chỗ duy nhất cần sửa), mở PR rồi merge.
4. Trên bản preview của PR, gửi thử một lần và kiểm tra hộp thư (cả thư mục spam).

Có JavaScript thì khách thấy thông báo thành công/lỗi bằng tiếng Việt ngay trên trang; tắt JavaScript thì form vẫn gửi được, nhưng khách sẽ thấy trang của Formspree (bằng tiếng Anh, có thể kèm bước xác minh chống spam) thay vì thông báo trên trang.

## Viết Bài chia sẻ mới

1. Tạo file Markdown trong `src/content/bai-chia-se/`. Tên file là đường dẫn của bài, viết không dấu, nối
   bằng gạch ngang: `dung-ai-viet-bien-ban-hop.md` → `/bai-chia-se/dung-ai-viet-bien-ban-hop/`.
2. Đầu file là frontmatter, đủ 4 mục:

   ```md
   ---
   title: Dùng AI viết biên bản họp trong 5 phút
   date: 2026-11-11
   tags: [Họp, Prompt mẫu]
   description: Một hai câu tóm tắt bài, hiện ở danh sách và Trang chủ.
   ---

   Nội dung bài viết bằng Markdown: ## tiêu đề mục, danh sách, **in đậm**, khối code...
   ```

   - `date`: ngày đăng, dạng `YYYY-MM-DD`. Danh sách xếp bài mới nhất lên đầu; Trang chủ hiện 3 bài mới nhất.
   - `tags`: ít nhất một thẻ chủ đề, viết có dấu như muốn hiển thị. Mỗi thẻ tự có trang lọc
     (`Prompt mẫu` → `/bai-chia-se/chu-de/prompt-mau/`). Dùng lại đúng tên thẻ cũ để bài vào chung một nhóm.
3. Chạy `npm run dev` để xem thử, rồi `npm test`: test báo lỗi nếu frontmatter thiếu mục hoặc bài dùng
   từ cần tránh trong `CONTEXT.md` (ví dụ "khóa học", "chuyên gia AI").
4. Commit và mở PR; merge vào `main` là bài lên web.

Bài mẫu: `src/content/bai-chia-se/tom-tat-email-dai-bang-ai.md`.
