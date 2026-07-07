/**
 * 英雄俗称表（社区叫法 → 搜索命中）。
 * 只收「不是称号/正式名子串」的俗称——champion-summary 的 name(称号)/description(正式名)
 * 已参与搜索，子串命中的不必重复收（如"剑圣"命中"无极剑圣"、"卡牌"命中"卡牌大师"）。
 * key = champion alias（英文，跨版本稳定），静态维护，随包分发。
 */
export const CHAMPION_NICKNAMES: Record<string, string[]> = {
  Aatrox: ['剑魔'],
  Ahri: ['狐狸'],
  Alistar: ['牛头'],
  Amumu: ['木乃伊'],
  Anivia: ['冰鸟'],
  Annie: ['火女'],
  AurelionSol: ['龙王'],
  Azir: ['沙皇', '黄鸡'],
  Blitzcrank: ['机器人'],
  Brand: ['火男'],
  Caitlyn: ['女警'],
  Cassiopeia: ['蛇女'],
  Chogath: ['大虫子'],
  Corki: ['飞机'],
  Darius: ['诺手'],
  Elise: ['蜘蛛'],
  Evelynn: ['寡妇'],
  Ezreal: ['EZ'],
  Fiddlesticks: ['稻草人'],
  Fizz: ['小鱼人'],
  Gragas: ['酒桶'],
  Hecarim: ['人马'],
  Heimerdinger: ['大头'],
  Illaoi: ['触手妈'],
  Irelia: ['刀妹'],
  JarvanIV: ['皇子'],
  Kalista: ['滑板鞋'],
  Kayle: ['天使'],
  Kennen: ['电耗子'],
  Khazix: ['螳螂'],
  KogMaw: ['大嘴'],
  LeeSin: ['瞎子'],
  Lissandra: ['冰女'],
  Lucian: ['奥巴马'],
  Malphite: ['石头人'],
  Maokai: ['大树'],
  MissFortune: ['女枪', 'MF'],
  MonkeyKing: ['猴子'],
  Nasus: ['狗头'],
  Nidalee: ['豹女'],
  Nocturne: ['梦魇'],
  Orianna: ['发条'],
  Rammus: ['龙龟'],
  Renekton: ['鳄鱼'],
  Rengar: ['狮子狗'],
  Sejuani: ['猪妹'],
  Shaco: ['小丑'],
  Shyvana: ['龙女'],
  Singed: ['炼金'],
  Skarner: ['蝎子'],
  Sona: ['琴女'],
  Soraka: ['奶妈'],
  Swain: ['乌鸦'],
  Syndra: ['球女'],
  TahmKench: ['蛤蟆'],
  Talon: ['男刀'],
  Tristana: ['小炮'],
  Tryndamere: ['蛮王'],
  Twitch: ['老鼠'],
  Vayne: ['VN'],
  Veigar: ['小法'],
  Velkoz: ['大眼'],
  Vladimir: ['吸血鬼'],
  Volibear: ['狗熊'],
  Warwick: ['狼人'],
  Yasuo: ['快乐风男'],
  Yorick: ['掘墓'],
  Yuumi: ['猫咪'],
  Zilean: ['时光老头']
}

/** 某英雄的俗称是否命中搜索词（大小写不敏感，包含匹配）。 */
export function nicknameMatches(alias: string, search: string): boolean {
  const nicknames = CHAMPION_NICKNAMES[alias]
  if (!nicknames) return false
  const s = search.toLowerCase()
  return nicknames.some((n) => n.toLowerCase().includes(s))
}
