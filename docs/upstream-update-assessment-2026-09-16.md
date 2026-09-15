# Đánh giá cập nhật NeoMind → HeraMind

**Ghi chú:** Đây là đánh giá trước tích hợp. Kết quả triển khai và kiểm chứng sau đó nằm tại [upstream-sync-0.9.24.md](upstream-sync-0.9.24.md).

Ngày kiểm tra: 16/09/2026. Phạm vi: lịch sử Git, diff mã nguồn, cấu hình, release và thử ghép văn bản ba phiên bản; chưa tích hợp hoặc chạy phiên bản mới.

## 1. Mốc đã xác minh

| Mốc | Giá trị |
|---|---|
| HeraMind hiện tại | `cbf789d3` — workspace 0.9.11, commit ngày 30/07/2026 |
| Lần đồng bộ được ghi trong lịch sử HeraMind | `178494c7` — sync NeoMind through v0.9.11 |
| Baseline NeoMind | `c7b54146518d3063f9a6b9aded7d87a0b43297a4` — tag `neomind/v0.9.11` tại local, trùng remote `v0.9.11` |
| NeoMind mục tiêu | `4c41362ad785b203e20cf49cc2dd3adb325d453e` — `upstream/main`, trùng remote `v0.9.24` lúc kiểm tra |
| Release | v0.9.24, published 15/09/2026 08:33:04 UTC; không phải prerelease |
| Changelog | ghi ngày 14/09/2026; khác ngày publish release |
| Chênh lệch upstream | 972 commit, 1.016 file, +91.098 / −24.764 dòng theo git diff |

Nguồn: [release v0.9.24](https://github.com/camthink-ai/NeoMind/releases/tag/v0.9.24), [diff v0.9.11…v0.9.24](https://github.com/camthink-ai/NeoMind/compare/v0.9.11...v0.9.24).

Hai lịch sử Git không có merge-base; repo không shallow. Không dùng `git merge upstream/main --allow-unrelated-histories` để đồng bộ ứng dụng. Nên dùng baseline nội dung v0.9.11 và ghép ba phiên bản sau khi ánh xạ namespace.

## 2. Thử ghép ba phiên bản

Đọc ba Git tree: baseline upstream v0.9.11, HeraMind HEAD, upstream v0.9.24. Chỉ chuẩn hóa ba chuỗi `NEOMIND/NeoMind/neomind` thành `HERAMIND/HeraMind/heramind` trong đường dẫn và nội dung không chứa NUL, sau đó dùng `git merge-file -p` trên file tạm.

| Phân loại | Số file |
|---|---:|
| Chỉ upstream thay đổi so với baseline; HeraMind còn khớp baseline | 847 |
| Cả hai thay đổi, ghép văn bản không xung đột | 79 |
| Cả hai đã có nội dung cuối tương đương | 51 |
| Xung đột văn bản | 28 |
| Upstream xóa nhưng HeraMind sửa | 7 |
| Xung đột file nhị phân | 4 |
| Chỉ HeraMind thay đổi/thêm | 126 |
| Không đổi ở cả ba | 595 |

Trong 1.016 file upstream thay đổi, 51 file đã tương đương; 965 file còn khác. Có 39 file cần xử lý riêng. Các số này là phép đối chiếu nội dung, không phải số conflict của một merge Git thật và không chứng minh tương thích nghiệp vụ. Phép so sánh lấy HEAD, không đưa việc xóa CLAUDE.md đang có trong working tree vào dữ liệu so sánh. File mode/symlink không được đánh giá bởi probe nội dung này.

## 3. Các cập nhật nên tiếp nhận

### Bảo mật và dữ liệu

- Chặn path traversal trong manifest extension và giới hạn giải nén; tham khảo `7bd9634c`, `0c8dab5f`, `386d0777` và các bản sửa tiếp theo.
- Guard SSRF dùng chung cho URL do thiết bị cung cấp: `88ee3c4b`. Cần kiểm thử riêng camera/LAN của HeraMind để không chặn nhầm nguồn ảnh hợp lệ.
- Session lưu bền và thu hồi khi đổi mật khẩu/xóa user; giới hạn thử đăng nhập; mặc định đóng tự đăng ký (`2e5d2d2e`, `dafc4b2d`, `274a543c`, `0a3d8098`).
- Mã hóa API key LLM tại nơi lưu trữ, dùng chung CryptoService ở core (`dfaf9cd5`). Dữ liệu cũ plaintext được giữ khả năng đọc và mã hóa khi save.
- Backup, giữ secret cùng database, schema stamp và guard chống mở dữ liệu mới bằng bản cũ (`b572e096`). Đây không phải migration framework.
- Đường dẫn database và encryption key được phân giải thống nhất; sửa tìm data dir cho CLI/server/desktop (`60eb46b6`, `62d08bf0`, `4c41362a`).

### IoT, rule và telemetry

- Giữ timestamp từ thiết bị, chuẩn hóa đơn vị, kiểm tra source/metric; sửa cửa sổ `hours`, aggregation, phân trang/cursor và dedup.
- Sửa transform trả string, state `for_duration`, hoàn cooldown khi toàn bộ action thất bại.
- Rule phát sự kiện vòng đời; data-push xử lý đầy hàng đợi và có log; `/api/metrics` quan sát HTTP và sự kiện bị bỏ.
- Các điểm này đụng trực tiếp MQTT analytics, bộ đếm, event list và transform của bodycam; cần kiểm thử cùng dữ liệu mẫu HeraMind.

### Agent và LLM

- Phát hiện vòng lặp kẹt, lý do dừng có kiểu dữ liệu, summary khi hết lượt, giới hạn thời gian một lượt chat.
- Sửa context budget, memory giữa phiên, tool calling trên endpoint tương thích OpenAI/llama.cpp và model nhỏ.
- Lượt chat tiếp tục trên server khi người dùng đổi trang hoặc mất kết nối; kết quả cuối được lưu vào lịch sử.
- Builtin llama.cpp, catalog và tải model; v0.9.24 mặc định bootstrap bật, cổng 29375. Đây là thay đổi vận hành có thể tải model và tăng RAM/dung lượng; cần cấu hình rõ khi port sang HeraMind.

### Extension, API và frontend

- SDK 0.6.4 → 0.7.1; SDK vẫn khai báo ABI 3 nhưng thêm raw FFI writer, segmented IPC và binary WS có negotiation. Phải kiểm thử runner + SDK + extension thực tế; không suy ra tương thích binary chỉ từ số ABI.
- `/api/docs`, `/api/docs/openapi.json`, schema dùng cho sinh client; sửa nhiều HTTP status và error envelope. Cần import frontend/CLI tương ứng cùng backend.
- IM bridges Telegram/Feishu, UI quản lý model và onboarding mới.
- Sidebar thay TopNav; chỉnh dashboard registry, Data Explorer, context indicator; React Router 6 → 7.
- Sửa Sparkline gọi hook có điều kiện (`999ac1bd`); CI bổ sung frontend lint/typecheck/unit tests, Rust fmt/clippy/workspace tests và desktop compile check.

## 4. Phần riêng HeraMind phải bảo toàn

- Namespace crate, binary, biến môi trường `HERAMIND_*`, đường dẫn dữ liệu, systemd, Tauri identifier, localStorage và API credential discovery.
- Logo, bảng màu, nội dung tiếng Việt, đường dẫn docs/release/update và marketplace. Không thay chuỗi mù quáng trong URL repo và protocol/FFI: endpoint hệ sinh thái có thể vẫn dùng tên NeoMind thật.
- `examples/bodycam/` và `examples/heracam-rv1126b/`, JSON device types, MQTT analytics và quy ước timestamp/counter.
- EventList, video rendering, dataMapping, eventProcessors và các test riêng.
- Agent prompt/intent hỗ trợ tiếng Việt. Upstream xóa `system_prompt.md` và thay bằng `system_prompt_slim.md`; cần chuyển hành vi sang prompt mới, không chỉ giữ file cũ.
- Upstream xóa `defaultConfigs.ts` và `usePageContext.ts`: chuyển phần đăng ký EventList và page context sang kiến trúc mới.

So sánh leaf key trong locale tiếng Anh của upstream v0.9.24 với locale Việt hiện tại thấy **436 key chưa có bản tương ứng**. Nhiều nhất: settings 147, plugins 104, dashboard-components 55, common 48, devices 30. Đây là thiếu key, chưa đánh giá chất lượng bản dịch hiện hữu.

## 5. Những điểm upstream chưa giải quyết

1. API key không có phân quyền thực: `a154f574` chỉ làm rõ `permissions` là thông tin, mọi API key được coi là quyền admin. Handler tạo key vẫn chỉ kiểm tra key hợp lệ. HeraMind muốn key giới hạn phải triển khai enforcement riêng.
2. Route `/api/images/*path` vẫn public. URL khó đoán không thay thế quyền truy cập hoặc hạn dùng.
3. `web/package.json` vẫn có script `build:check` có thể báo OK khi tsc thất bại.
4. CI frontend dùng `lint:ci --quiet`, chỉ chặn error; không có nghĩa warning đã sạch.

## 6. Kế hoạch tích hợp đề xuất

Mục tiêu: nhánh tích hợp HeraMind dựa trên snapshot NeoMind v0.9.24, giữ các thay đổi riêng; chưa phát hành theo từng trạng thái trung gian.

1. Tạo branch/worktree riêng từ HeraMind HEAD. Ghi provenance gồm upstream baseline + target SHA, quy tắc namespace và danh sách file riêng. Bảo toàn thay đổi CLAUDE.md hiện có.
2. Dựng snapshot upstream đã ánh xạ đường dẫn; ghép ba phiên bản. Chia commit review theo core/storage/auth, devices/rules/push, agent/LLM, extension, API/CLI, frontend/i18n, deploy/CI; kiểm tra dependencies giữa nhóm.
3. Giải quyết 39 file cần xử lý riêng; rà cả 79 file ghép văn bản tự động. Ưu tiên agent streaming, auth_users, server/types, CLI dispatch, API client và dashboard registry.
4. Chuyển đầy đủ tính năng bodycam/EventList và tiếng Việt sang cấu trúc frontend/prompt mới. Đồng bộ phiên bản root workspace, npm, Tauri, SDK và hai Cargo.lock bằng công cụ tương ứng.
5. Chạy kiểm thử bên dưới trên dữ liệu mẫu/snapshot. Chỉ sau khi đạt mới chuẩn bị bản phát hành HeraMind.

### Kiểm tra chấp nhận

- Backend: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`.
- Frontend: `npm ci`, `npm run lint:ci` (nếu đã nhập script), `npx tsc --noEmit`, `npm test -- --run`, `npm run build`; kiểm tra riêng cảnh báo vòng bundle và lỗi build:check.
- Desktop: `cargo check --locked` trong `web/src-tauri`; smoke khởi động/shutdown server + runner, dữ liệu và LAN setting.
- MQTT/bodycam: ingest sample event → timestamp/counter đúng → transform → lưu/query telemetry → dashboard/EventList → rule/data-push.
- Chat: tiếng Việt, gọi tool thực, cancel, đổi trang, reconnect, lịch sử cuối, ngân sách context nhỏ.
- Extension: binary hiện dùng, capability IPC, crash/restart, text fallback và binary WS negotiation.
- Auth/data: login/logout/restart, reset mật khẩu, custom data-dir, giữ secret, key cũ, backup/restore trên bản sao.
- API: status/error envelope, OpenAPI, client CLI/frontend, chia sẻ dashboard và truy cập ảnh.

### Dữ liệu và rollback

Trước lần chạy đầu trên dữ liệu thật, tạo bản sao nhất quán khi server đã dừng, gồm toàn bộ data dir và secret. Giữ bản sao trước nâng cấp để rollback cả binary lẫn dữ liệu. HeraMind 0.9.11 chưa có schema guard mới nên không thể trông chờ nó tự từ chối database đã được bản mới ghi.

Release note v0.9.24 cảnh báo riêng đường nâng từ NeoMind 0.9.21–0.9.23 với custom data-dir có thể phải tạo lại API key do lỗi ghép đường dẫn secret. Chưa có bằng chứng HeraMind hiện tại mắc đúng regression đó; phải thử migration trên bản sao thay vì mặc định xóa/tạo lại key.

## 7. Trạng thái kiểm tra

Đã fetch upstream, xác minh release/tag/SHA, đọc diff và chạy probe ghép văn bản; chưa merge/cherry-pick, chưa nâng dependency, chưa khởi chạy code upstream hoặc migration trên data hiện hữu.

Baseline HeraMind đã kiểm tra ở lượt phân tích trước: cargo check workspace đạt; frontend build đạt (có cảnh báo); 139/139 unit test đạt; lint có 30 lỗi và 1.165 cảnh báo; 146 case eval hợp lệ về cấu trúc. Đây không phải kết quả test v0.9.24.

## Phụ lục: file cần xử lý riêng

- `.gitignore` — text_conflict
- `Cargo.lock` — text_conflict
- `README.md` — text_conflict
- `README.zh.md` — text_conflict
- `crates/heramind-agent/src/agent/mod.rs` — text_conflict
- `crates/heramind-agent/src/agent/streaming/stream_core.rs` — text_conflict
- `crates/heramind-agent/src/llm.rs` — text_conflict
- `crates/heramind-agent/src/prompts/system_prompt.md` — structural_conflict
- `crates/heramind-api/src/auth_users.rs` — text_conflict
- `crates/heramind-api/src/handlers/skills.rs` — text_conflict
- `crates/heramind-api/src/server/types.rs` — text_conflict
- `crates/heramind-cli-ops/src/dispatch/commands.rs` — text_conflict
- `crates/heramind-cli-ops/src/dispatch/handlers.rs` — text_conflict
- `crates/heramind-cli-ops/tests/skill_cli_drift.rs` — text_conflict
- `crates/heramind-cli/src/main.rs` — text_conflict
- `crates/heramind-cli/src/self_update.rs` — text_conflict
- `crates/heramind-data-push/src/scheduler.rs` — text_conflict
- `crates/heramind-extension-sdk/README.md` — text_conflict
- `docs/img/chat.png` — structural_conflict
- `docs/img/devices.png` — structural_conflict
- `docs/img/mobile_web.png` — binary_conflict
- `web/public/logo-dark.png` — binary_conflict
- `web/public/logo-light.png` — binary_conflict
- `web/public/logo-square.png` — binary_conflict
- `web/src/components/dashboard/registry/defaultConfigs.ts` — structural_conflict
- `web/src/components/layout/TopNav.tsx` — structural_conflict
- `web/src/components/shared/HoneycombBackground.tsx` — structural_conflict
- `web/src/design-system/tokens/color.ts` — text_conflict
- `web/src/hooks/usePageContext.ts` — structural_conflict
- `web/src/i18n/locales/en/common.json` — text_conflict
- `web/src/i18n/locales/en/dashboard-components.json` — text_conflict
- `web/src/i18n/locales/zh/common.json` — text_conflict
- `web/src/i18n/locales/zh/dashboard-components.json` — text_conflict
- `web/src/index.css` — text_conflict
- `web/src/lib/api.ts` — text_conflict
- `web/src/pages/login.tsx` — text_conflict
- `web/src/pages/settings/AboutTab.tsx` — text_conflict
- `web/src/pages/setup/SetupBackground.tsx` — text_conflict
- `web/src/pages/setup/SetupHeader.tsx` — text_conflict
