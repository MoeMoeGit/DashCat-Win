# 发行

仓库同时保留 `.github/workflows/build.yml` 与 `release.yml`，都支持 v* 标签与手动触发；build.yml 还包含 MSIX 路径。普通 main push 不执行这些工作流。

AppxManifest.xml 与图标已经存在，但签名、安装、升级及商店上架未在本轮验证。两个发行工作流存在重复，后续发布前应核对产物命名、权限和重复 Release 动作。
