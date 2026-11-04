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

Chỗ còn `TODO(owner)` là nội dung thật người sáng lập cần tự điền (`src/site.ts`, trang Giới thiệu).
