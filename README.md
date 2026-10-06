# bevy_flash WebGPU showcase

一个 Bevy 应用、一个 canvas，展示离线 VAB 动画和矢量 UI。HTML 负责导航和控制面板，Bevy 负责画布中的动画、UI 图形和原生按钮。只支持 WebGPU，不包含 WebGL 回退。

## 展示页

| 页 | 内容 | 参考库例子 |
|---|---|---|
| 动画播放 | 素材/动作切换、loop/once/hold/terminal、动作链、fallback、暂停、速度、定位、事件、25%–400% 展示缩放 | show_demo |
| 皮肤切换 | 从资产枚举槽位与变体，逐槽选择 | wu_kong_skin |
| 矢量 UI | 静态名字框与动态 sparkles，三个尺寸节点、等比缩放和共享栅格缓存 | ui_graphics / animated_ui |
| 原生按钮 | up/over/down、禁用、按下反馈；hit 不作为外观 | ui_buttons |

只加载当前页的资产；切页移除旧页的实体和主资产句柄。库的闲置 UI/滤镜缓存遵循自身预算，并非切页立即释放全部 GPU 资源。加载失败会显示错误和重试按钮。

## 本地依赖

同级目录需要接口匹配的 bevy 与 bevy_flash，后者依赖 vatf。引擎/库版本组合以 bevy_flash 的 Compatibility 表为准。Rust 工具链应满足 Bevy 的 rust-version。

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked --root .tools/wasm-bindgen
./build-web.ps1
```

wasm-bindgen CLI 的版本须与 Cargo.lock 的 wasm-bindgen 一致；锁文件更新后同步工具版本。脚本优先使用项目内工具，不替换全局安装。

构建脚本编译 release WASM、生成 JS 接口，再把页面和资产复制到 dist。开发检查可用 `./build-web.ps1 -DebugBuild`。为了保留动画和滤镜的执行性能，release 使用 opt-level=3；不优先追求最小文件。

用任意静态 HTTP 服务托管 dist。例如已经安装 Python 时：

```powershell
python -m http.server 8080 --directory dist
```

打开 `http://localhost:8080`。发布时使用 HTTPS，上传 dist 的全部内容；部署到网站子目录也使用相对资源路径。不要直接打开 file://，也不要在浏览器运行 SWF 编译器。本地构建和预览不等于发布；GitHub 自动部署见下一节。

## GitHub Pages 自动发布

`.github/workflows/pages.yml` 使用 GitHub Actions 构建 release WASM，再将 dist 发布到 GitHub Pages。PR 只检查构建；推送 main 或在 main 手动运行 WebGPU showcase 工作流会部署。无需提交 dist，也无需维护 gh-pages 分支。

首次启用：

1. 将示例代码、Cargo.lock、七个 VAB 资产和工作流提交到此仓库。
2. 先将 bevy_flash 的浏览器计时与 WebGPU 着色器修复，以及 vatf 的未使用 git 依赖移除推送到对应仓库。CI 无法读取本地未提交的修改。
3. 在此仓库 **Settings → Pages → Build and deployment → Source** 选择 **GitHub Actions**。
4. 推送 main，或在 Actions 中选择 **WebGPU showcase → Run workflow**（main）。成功后可在部署作业打开页面地址；默认地址为 https://aojiaoxiaolinlin.github.io/bevy_flash_example/。

工作流在 Ubuntu 上按同级目录布局检出 bevy、bevy_flash、vatf；build-web.ps1 支持 PowerShell 7 的 Windows/Linux 环境。Bevy revision 和 Rust 工具链固定在工作流 env 中，wasm-bindgen CLI 版本自动读取 Cargo.lock，构建使用 --locked。更新 Bevy 时同步调整 BEVY_REF、RUST_VERSION 和锁文件。

开发期间 FLASH_REF / VATF_REF 跟踪 main，实际提交号会记录在每次运行的摘要中。正式发布前将这两个 ref 也固定为经过验证的 commit SHA；VAB 尚未发布稳定格式，更新编译器时要同步资产。依赖仓库推送不会自动触发此仓库的部署，可手动运行工作流。

构建缓存包括 Cargo、WASM 编译结果与匹配的 wasm-bindgen CLI。只有部署作业具有 pages:write 和 id-token:write，PR 构建没有发布权限。页面继续仅支持 WebGPU；GitHub Pages 的 HTTPS 满足其安全上下文要求。Pages 发布流程参考 [GitHub 官方说明](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)。

## 资产

assets/ 包含整理后的 VAB，无需运行时依赖库项目的 assets。sources.md 记录来源与刷新命令；所有素材仅用于开发展示，需自行确认原作品的使用授权。

动态背景约 10 MiB，角色和换肤素材也有数 MiB，按页请求可降低首屏下载量。服务器可为 WASM/JS/VAB 配置压缩与缓存；更新资源时应避免混用不同编译器产生的工作格式。

## 代码与验证

src/catalog.rs 定义素材目录；bridge.rs 是浏览器命令/状态接口；bounds.rs 固定取景范围；demos.rs 管理页面、加载与控制。取景只在加载时测量各动作帧的联合范围，窗口变化只调整展示缩放，不按当前帧重设原点。动画页和换肤页提供 25%–400% 的展示缩放，100% 表示自动适配后的尺寸；资源倍率显示最终 Transform 缩放。切换素材恢复 100%，放大围绕固定取景中心，超出画布的部分会被裁切，重置适配可恢复全景。画布背景采用 Flash 常用的 sRGB (102, 102, 102)。

```powershell
cargo check --target wasm32-unknown-unknown
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

原生验证用 `cargo run`：1–4 切页，Space 暂停；完整控制面板在网页中。网页验收应检查四页切换、动作完成回 fallback、终态锁定与重置、皮肤、UI 循环、按钮 hover/press/禁用、窗口缩放和加载失败重试。

诊断面板默认关闭，显示平滑应用帧率，包含浏览器调度开销，不声称为 GPU 滤镜耗时。精确屏幕取样混合不在当前库范围内，Lighten 使用 Max 近似。

动画页和换肤页使用 `CompositingSpace::Srgb`，与 Flash 的合成方式一致；矢量 UI 和按钮页使用默认线性合成，因为当前 Bevy 原生 UI shader 输出线性颜色，不能直接写入 sRGB 合成目标。切页时切换相机配置，公开 UI 输出纹理仍保持 `Rgba8UnormSrgb`。
