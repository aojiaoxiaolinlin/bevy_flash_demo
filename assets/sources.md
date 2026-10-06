# 演示资产来源

所有 VAB 原样复制自接口匹配的 bevy_flash/assets，不下载外部素材，也不修改其内容。

| 本项目 | 对应库素材 |
|---|---|
| animations/spirit2159src.vab | spirit2159src.vab |
| animations/spirit3021src.vab | spirit3021src.vab |
| animations/luo_ke_lu.vab | luo_ke_lu.vab |
| animations/wu_kong.vab | wu_kong.vab |
| ui/nameplate3.vab | nameplate3.vab |
| ui/background551284.vab | background551284.vab |
| ui/login.vab | login.vab |

源 SWF 用 vatf 转换：动画不加 UI 选项，名字框和按钮使用 --ui，动态背景使用 --ui-animated。源文件位置及导出约定见 vatf/fixtures/README.md；没有对应源码的演示资产须先找回源 SWF，不能仅修改 VAB 版本号。

编译器行为或工作格式变化后，应重新生成库资产并同步复制此列表中的文件。角色素材 3021 的源文件需要制作端保留；此演示不在浏览器编译它。

这些游戏素材仅用于开发展示；复制不赋予原作品的授权。
