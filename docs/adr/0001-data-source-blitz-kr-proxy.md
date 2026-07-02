# 数据源用 Blitz Datalake 外服数据，作国服代理

用户玩国服，但海克斯大乱斗的真国服统计数据不存在——Riot API 不暴露增强数据（developer-relations #1154），腾讯国服无开放统计接口，市面所有可取源（Blitz/arammayhem/aramkit/aramgg）全是外服(KR/全球)。故主源选 Blitz Datalake（`datalake.v2.iesdev.com/graphql`，实测免鉴权、返回海克斯/出装/三连组合胜率），作为国服代理使用。

## Consequences

- UI 必须显眼标注「数据来源 KR / 版本 X」，让用户对国服版本滞后自行判断。
- 代理成立的依据：海克斯**相对优先级排序**由游戏机制驱动、跨服可迁移；绝对胜率数值在国服版本滞后时会有偏差，但排序稳定。
- 后续（v1 后并行）：Blitz + arammayhem/op.gg 双外服源容灾（防单源抓取失败，非地区交叉）；深探海斗小助手微信小程序是否真有国服数据（概率低）。
