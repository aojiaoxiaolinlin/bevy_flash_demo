# bevy_flash WebGPU demo

一个 Bevy 应用展示 VAB 动画、命名皮肤和矢量 UI。导航和控制面板全部使用
Bevy UI，桌面与浏览器使用相同的 Rust 展示逻辑。JavaScript 只负责启动 WASM、
提示 WebGPU 不可用或初始化失败，并关闭右键菜单。仅支持 WebGPU。

## 操作

- Anim：左右箭头切换资源，点击动作名称播放一次，结束后自动返回 fallback。
- 动作右侧 Set fallback：指定返回动作，当前 fallback 显示为蓝色 Fallback。设置不会打断正在播放的动作。首次加载优先选择 idle，否则使用首个动作。
- 当前动作名称用绿色标记；fallback 用蓝色标记，底部始终显示当前播放与返回动作。
- Skins：悟空的每个皮肤槽都有切换按钮，点击循环选择其具名变体。
- UI：静态名字框和动态 sparkles，默认等比显示，展示不同尺寸的缓存复用。
- Buttons：原生按钮的 hover/press 状态。
- Zoom + / - 或画布滚轮：25%–400% 展示缩放；100% 为自动适配后的尺寸。
- 动画画布右键拖拽或方向键：移动相机；Fit view 恢复完整取景。
- 控制面板上滚轮：滚动动作/皮肤列表。Space 暂停/继续，1–4 切换展示页。

不提供逐帧定位、播放速度、终态和事件列表等调试控件。加载与错误信息显示在
Bevy 控制面板内，Reload resource 可重试。切换资源恢复适配和相机位置。

固定取景在加载时计算全部动作的联合范围，不随当前帧重新对齐，因此播放不会
因包围盒变化而抖动。世界动画使用 sRGB 合成相机；Bevy UI 使用独立的线性
合成相机，控制面板和导出的 UI 不受 Flash 合成空间影响。

## 构建

依赖来自 crates.io（Bevy 0.20）和 Git（bevy_flash / vatf），无需同级源码目录。
`rust-toolchain.toml` 固定 Rust 1.97.1，`Cargo.lock` 固定依赖修订。

```powershell
rustup target add wasm32-unknown-unknown
cargo run
cargo check --locked --target wasm32-unknown-unknown
cargo clippy --all-targets -- -D warnings
```

浏览器构建需要与 Cargo.lock 中版本一致的 wasm-bindgen-cli：

```powershell
cargo install wasm-bindgen-cli --version <Cargo.lock 中的 wasm-bindgen 版本> --locked --root .tools/wasm-bindgen
./build-web.ps1
python -m http.server 8080 --directory dist
```

打开 `http://localhost:8080`；线上必须使用 HTTPS。页面与资产使用相对路径，
可以部署在 GitHub Pages 子目录。运行时只加载打包的 VAB，不运行 SWF 编译器。

## GitHub Pages

`.github/workflows/pages.yml` 使用锁定依赖构建 release WASM 并发布 dist。
PR 只构建，main 推送或手动运行会部署。仓库 Settings → Pages → Source
选择 GitHub Actions。无需检出 Bevy、渲染库或编译器的同级源码仓库。
wasm-bindgen-cli 版本自动读取 Cargo.lock；更新 Git 依赖后需提交锁文件。

## 代码与资源

- `src/catalog.rs`：展示资源目录。
- `src/controls.rs`：Bevy 控件、动作消息、缩放和平移。
- `src/demos.rs`：切页、按需加载、播放和皮肤状态。
- `src/bounds.rs`：固定联合取景范围。

assets 包含展示用 VAB，来源及刷新方式见 assets/sources.md。仅加载当前页资产。
Lighten 使用库内的 Max 近似，精确屏幕取样混合仍未实现。

面板采用分区布局：页签和资源切换在上方，动作/皮肤列表独立滚动，暂停、缩放和重置固定在底部。字体随 assets/fonts 一同打包。

验证：cargo test 运行原生控件测试；cargo test -- --include-ignored --test-threads=1 额外运行 GPU 合成与真实资源 fallback 回归（需要 GPU）。
