use std::{fs::File, path::PathBuf};

use anyhow::{bail, Context};
use mcvm_shared::{instance::Side, versions::VersionInfo};
use serde::Deserialize;
use zip::ZipArchive;

use super::files::paths::Paths;

pub mod game_jar {
	use std::io::BufReader;

	use mcvm_shared::versions::VersionPattern;

	use super::*;

	/// Format for the version.json file in the game jar
	#[derive(Deserialize)]
	pub struct VersionJson {
		/// The Minecraft data version number for this version, used for worlds, options.txt, etc.
		#[serde(rename = "world_version")]
		pub data_version: i32,
	}

	/// Extract the version.json file from the game jar
	pub fn extract_version_json(mc_version: &str, paths: &Paths) -> anyhow::Result<VersionJson> {
		let path = get_existing_path(mc_version, paths)
			.context("Failed to get a game jar to extract the version.json file from")?;
		let file = File::open(path).context("Failed to open game jar file")?;
		let file = BufReader::new(file);
		let mut zip = ZipArchive::new(file).context("Failed to create zip archive")?;
		let file = zip
			.by_name("version.json")
			.context("Failed to find file in game jar")?;
		serde_json::from_reader(file).context("Failed to parse version.json")
	}

	/// Extract the version.json file optionally, only if the version has the file in there
	pub fn extract_version_json_optional(
		version_info: &VersionInfo,
		paths: &Paths,
	) -> anyhow::Result<Option<VersionJson>> {
		if VersionPattern::After(String::from("18w47b")).matches_info(version_info) {
			Ok(Some(extract_version_json(&version_info.version, paths)?))
		} else {
			Ok(None)
		}
	}

	/// Gets the path to a stored game jar file
	pub fn get_path(side: Side, version: &str, paths: &Paths) -> PathBuf {
		let side_str = side.to_string();
		paths.jars.join(format!("{version}_{side_str}.jar"))
	}

	/// Get the path to either the client or server jar. If the client path doesn't exist,
	/// then the server path will be chosen
	pub fn get_existing_path(mc_version: &str, paths: &Paths) -> anyhow::Result<PathBuf> {
		let mut path = get_path(Side::Client, mc_version, paths);
		if !path.exists() {
			path = get_path(Side::Server, mc_version, paths);
		}
		if !path.exists() {
			bail!("An existing game jar for this Minecraft version does not exist");
		}

		Ok(path)
	}
}

/// Get the game data version either from the game jar or the known map
pub fn get_data_version(version_info: &VersionInfo, paths: &Paths) -> anyhow::Result<Option<i32>> {
	if let Some(version_json) = game_jar::extract_version_json_optional(version_info, paths)
		.context("Failed to extract version.json")?
	{
		Ok(Some(version_json.data_version))
	} else {
		Ok(get_old_data_version(&version_info.version))
	}
}

/// Get the data version for versions before 18w47b that do not include it in the version.json.
/// Versions before 15w32a do not have a data version
pub fn get_old_data_version(mc_version: &str) -> Option<i32> {
	match mc_version {
		"23w33a" => Some(3570),
		"23w32a" => Some(3569),
		"23w31a" => Some(3467),
		"1.20.1" => Some(3465),
		"1.20.1 Release Candidate 1" => Some(3464),
		"1.20" => Some(3463),
		"1.20 Release Candidate 1" => Some(3462),
		"1.20 Pre-release 7" => Some(3461),
		"1.20 Pre-release 6" => Some(3460),
		"1.20 Pre-release 5" => Some(3458),
		"1.20 Pre-release 4" => Some(3457),
		"1.20 Pre-release 3" => Some(3456),
		"1.20 Pre-release 2" => Some(3455),
		"1.20 Pre-release 1" => Some(3454),
		"23w18a" => Some(3453),
		"23w17a" => Some(3452),
		"23w16a" => Some(3449),
		"23w14a" => Some(3445),
		"23w13a" => Some(3443),
		"23w12a" => Some(3442),
		"1.19.4" => Some(3337),
		"1.19.4 Release Candidate 3" => Some(3336),
		"1.19.4 Release Candidate 2" => Some(3335),
		"1.19.4 Release Candidate 1" => Some(3334),
		"1.19.4 Pre-release 4" => Some(3333),
		"1.19.4 Pre-release 3" => Some(3332),
		"1.19.4 Pre-release 2" => Some(3331),
		"1.19.4 Pre-release 1" => Some(3330),
		"23w07a" => Some(3329),
		"23w06a" => Some(3326),
		"23w05a" => Some(3323),
		"23w04a" => Some(3321),
		"23w03a" => Some(3320),
		"1.19.3" => Some(3218),
		"1.19.3 Release Candidate 3" => Some(3217),
		"1.19.3 Release Candidate 2" => Some(3216),
		"1.19.3 Release Candidate 1" => Some(3215),
		"1.19.3 Pre-release 3" => Some(3213),
		"1.19.3 Pre-release 2" => Some(3212),
		"1.19.3 Pre-release 1" => Some(3211),
		"22w46a" => Some(3210),
		"22w45a" => Some(3208),
		"22w44a" => Some(3207),
		"22w43a" => Some(3206),
		"22w42a" => Some(3205),
		"1.19.2" => Some(3120),
		"1.19.2 Release Candidate 2" => Some(3119),
		"1.19.2 Release Candidate 1" => Some(3118),
		"1.19.1" => Some(3117),
		"1.19.1 Release Candidate 3" => Some(3116),
		"1.19.1 Release Candidate 2" => Some(3115),
		"1.19.1 Pre-release 6" => Some(3114),
		"1.19.1 Pre-release 5" => Some(3113),
		"1.19.1 Pre-release 4" => Some(3112),
		"1.19.1 Pre-release 3" => Some(3111),
		"1.19.1 Pre-release 2" => Some(3110),
		"1.19.1 Release Candidate 1" => Some(3109),
		"1.19.1 Pre-release 1" => Some(3107),
		"22w24a" => Some(3106),
		"1.19" => Some(3105),
		"1.19 Release Candidate 2" => Some(3104),
		"1.19 Release Candidate 1" => Some(3103),
		"1.19 Pre-release 5" => Some(3102),
		"1.19 Pre-release 4" => Some(3101),
		"1.19 Pre-release 3" => Some(3100),
		"1.19 Pre-release 2" => Some(3099),
		"1.19 Pre-release 1" => Some(3098),
		"22w19a" => Some(3096),
		"22w18a" => Some(3095),
		"22w17a" => Some(3093),
		"22w16b" => Some(3092),
		"22w16a" => Some(3091),
		"22w15a" => Some(3089),
		"22w14a" => Some(3088),
		"22w13a" => Some(3085),
		"22w12a" => Some(3082),
		"22w11a" => Some(3080),
		"Deep Dark Experimental Snapshot 1" => Some(3066),
		"1.18.2" => Some(2975),
		"1.18.2 Release Candidate 1" => Some(2974),
		"1.18.2 Pre-release 3" => Some(2973),
		"1.18.2 Pre-release 2" => Some(2972),
		"1.18.2 Pre-release 1" => Some(2971),
		"22w07a" => Some(2969),
		"22w06a" => Some(2968),
		"22w05a" => Some(2967),
		"22w03a" => Some(2966),
		"1.18.1" => Some(2865),
		"1.18.1 Release Candidate 3" => Some(2864),
		"1.18.1 Release Candidate 2" => Some(2863),
		"1.18.1 Release Candidate 1" => Some(2862),
		"1.18.1 Pre-release 1" => Some(2861),
		"1.18" => Some(2860),
		"1.18 Release Candidate 4" => Some(2859),
		"1.18 Release Candidate 3" => Some(2858),
		"1.18 Release Candidate 2" => Some(2857),
		"1.18 Release Candidate 1" => Some(2856),
		"1.18 Pre-release 8" => Some(2855),
		"1.18 Pre-release 7" => Some(2854),
		"1.18 Pre-release 6" => Some(2853),
		"1.18 Pre-release 5" => Some(2851),
		"1.18 Pre-release 4" => Some(2850),
		"1.18 Pre-release 3" => Some(2849),
		"1.18 Pre-release 2" => Some(2848),
		"1.18 Pre-release 1" => Some(2847),
		"21w44a" => Some(2845),
		"21w43a" => Some(2844),
		"21w42a" => Some(2840),
		"21w41a" => Some(2839),
		"21w40a" => Some(2838),
		"21w39a" => Some(2836),
		"21w38a" => Some(2835),
		"21w37a" => Some(2834),
		"1.18 experimental snapshot 7" => Some(2831),
		"1.18 experimental snapshot 6" => Some(2830),
		"1.18 experimental snapshot 5" => Some(2829),
		"1.18 experimental snapshot 4" => Some(2828),
		"1.18 experimental snapshot 3" => Some(2827),
		"1.18 experimental snapshot 2" => Some(2826),
		"1.18 Experimental Snapshot 1" => Some(2825),
		"1.17.1" => Some(2730),
		"1.17.1 Release Candidate 2" => Some(2729),
		"1.17.1 Release Candidate 1" => Some(2728),
		"1.17.1 Pre-release 3" => Some(2727),
		"1.17.1 Pre-release 2" => Some(2726),
		"1.17.1 Pre-release 1" => Some(2725),
		"1.17" => Some(2724),
		"1.17 Release Candidate 2" => Some(2723),
		"1.17 Release Candidate 1" => Some(2722),
		"1.17 Pre-release 5" => Some(2721),
		"1.17 Pre-release 4" => Some(2720),
		"1.17 Pre-release 3" => Some(2719),
		"1.17 Pre-release 2" => Some(2718),
		"1.17 Pre-release 1" => Some(2716),
		"21w20a" => Some(2715),
		"21w19a" => Some(2714),
		"21w18a" => Some(2713),
		"21w17a" => Some(2712),
		"21w16a" => Some(2711),
		"21w15a" => Some(2709),
		"21w14a" => Some(2706),
		"21w13a" => Some(2705),
		"21w11a" => Some(2703),
		"21w10a" => Some(2699),
		"21w08b" => Some(2698),
		"21w08a" => Some(2697),
		"21w07a" => Some(2695),
		"21w06a" => Some(2694),
		"21w05b" => Some(2692),
		"21w05a" => Some(2690),
		"21w03a" => Some(2689),
		"20w51a" => Some(2687),
		"20w49a" => Some(2685),
		"20w48a" => Some(2683),
		"20w46a" => Some(2682),
		"20w45a" => Some(2681),
		"Combat Test 8c" => Some(2707),
		"Combat Test 8b" => Some(2706),
		"Combat Test 8" => Some(2705),
		"Combat Test 7c" => Some(2704),
		"Combat Test 7b" => Some(2703),
		"Combat Test 7" => Some(2702),
		"Combat Test 6" => Some(2701),
		"1.16.5" => Some(2586),
		"1.16.5 Release Candidate 1" => Some(2585),
		"1.16.4" => Some(2584),
		"1.16.4 Release Candidate 1" => Some(2583),
		"1.16.4 Pre-release 2" => Some(2582),
		"1.16.4 Pre-release 1" => Some(2581),
		"1.16.3" => Some(2580),
		"1.16.3 Release Candidate 1" => Some(2579),
		"1.16.2" => Some(2578),
		"1.16.2 Release Candidate 2" => Some(2577),
		"1.16.2 Release Candidate 1" => Some(2576),
		"1.16.2 Pre-release 3" => Some(2575),
		"1.16.2 Pre-release 2" => Some(2574),
		"1.16.2 Pre-release 1" => Some(2573),
		"20w30a" => Some(2572),
		"20w29a" => Some(2571),
		"20w28a" => Some(2570),
		"20w27a" => Some(2569),
		"1.16.1" => Some(2567),
		"1.16" => Some(2566),
		"1.16 Release Candidate 1" => Some(2565),
		"1.16 Pre-release 8" => Some(2564),
		"1.16 Pre-release 7" => Some(2563),
		"1.16 Pre-release 6" => Some(2562),
		"1.16 Pre-release 5" => Some(2561),
		"1.16 Pre-release 4" => Some(2560),
		"1.16 Pre-release 3" => Some(2559),
		"1.16 Pre-release 2" => Some(2557),
		"1.16 Pre-release 1" => Some(2556),
		"20w22a" => Some(2555),
		"20w21a" => Some(2554),
		"20w20b" => Some(2537),
		"20w20a" => Some(2536),
		"20w19a" => Some(2534),
		"20w18a" => Some(2532),
		"20w17a" => Some(2529),
		"20w16a" => Some(2526),
		"20w15a" => Some(2525),
		"20w14a" => Some(2524),
		"20w13b" => Some(2521),
		"20w13a" => Some(2520),
		"20w12a" => Some(2515),
		"20w11a" => Some(2513),
		"20w10a" => Some(2512),
		"20w09a" => Some(2510),
		"20w08a" => Some(2507),
		"20w07a" => Some(2506),
		"Snapshot 20w06a" => Some(2504),
		"Combat Test 5" => Some(2321),
		"Combat Test 4" => Some(2320),
		"1.15.2" => Some(2230),
		"1.15.2 Pre-release 2" => Some(2229),
		"1.15.2 Pre-Release 1" => Some(2228),
		"1.15.1" => Some(2227),
		"1.15.1 Pre-release 1" => Some(2226),
		"1.15" => Some(2225),
		"1.15 Pre-release 7" => Some(2224),
		"1.15 Pre-release 6" => Some(2223),
		"1.15 Pre-release 5" => Some(2222),
		"1.15 Pre-release 4" => Some(2221),
		"1.15 Pre-release 3" => Some(2220),
		"1.15 Pre-Release 2" => Some(2219),
		"1.15 Pre-release 1" => Some(2218),
		"19w46b" => Some(2217),
		"19w46a" => Some(2216),
		"19w45b" => Some(2215),
		"19w45a" => Some(2214),
		"19w44a" => Some(2213),
		"19w42a" => Some(2212),
		"19w41a" => Some(2210),
		"19w40a" => Some(2208),
		"19w39a" => Some(2207),
		"19w38b" => Some(2206),
		"19w38a" => Some(2205),
		"19w37a" => Some(2204),
		"19w36a" => Some(2203),
		"19w35a" => Some(2201),
		"19w34a" => Some(2200),
		"Combat Test 3" => Some(2069),
		"Combat Test 2" => Some(2068),
		"1.14.3 - Combat Test" => Some(2067),
		"1.14.4" => Some(1976),
		"1.14.4 Pre-Release 7" => Some(1975),
		"1.14.4 Pre-Release 6" => Some(1974),
		"1.14.4 Pre-Release 5" => Some(1973),
		"1.14.4 Pre-Release 4" => Some(1972),
		"1.14.4 Pre-Release 3" => Some(1971),
		"1.14.4 Pre-Release 2" => Some(1970),
		"1.14.4 Pre-Release 1" => Some(1969),
		"1.14.3" => Some(1968),
		"1.14.3 Pre-Release 4" => Some(1967),
		"1.14.3 Pre-Release 3" => Some(1966),
		"1.14.3 Pre-Release 2" => Some(1965),
		"1.14.3 Pre-Release 1" => Some(1964),
		"1.14.2" => Some(1963),
		"1.14.2 Pre-Release 4" => Some(1962),
		"1.14.2 Pre-Release 3" => Some(1960),
		"1.14.2 Pre-Release 2" => Some(1959),
		"1.14.2 Pre-Release 1" => Some(1958),
		"1.14.1" => Some(1957),
		"1.14.1 Pre-Release 2" => Some(1956),
		"1.14.1 Pre-Release 1" => Some(1955),
		"1.14" => Some(1952),
		"1.14 Pre-Release 5" => Some(1951),
		"1.14 Pre-Release 4" => Some(1950),
		"1.14 Pre-Release 3" => Some(1949),
		"1.14 Pre-Release 2" => Some(1948),
		"1.14 Pre-Release 1" => Some(1947),
		"19w14b" => Some(1945),
		"19w14a" => Some(1944),
		"19w13b" => Some(1943),
		"19w13a" => Some(1942),
		"19w12b" => Some(1941),
		"19w12a" => Some(1940),
		"19w11b" => Some(1938),
		"19w11a" => Some(1937),
		"19w09a" => Some(1935),
		"19w08b" => Some(1934),
		"19w08a" => Some(1933),
		"19w07a" => Some(1932),
		"19w06a" => Some(1931),
		"19w05a" => Some(1930),
		"19w04b" => Some(1927),
		"19w04a" => Some(1926),
		"19w03c" => Some(1924),
		"19w03b" => Some(1923),
		"19w03a" => Some(1922),
		"19w02a" => Some(1921),
		"18w50a" => Some(1919),
		"18w49a" => Some(1916),
		"18w48b" => Some(1915),
		"18w48a" => Some(1914),
		"18w47b" => Some(1913),
		"18w47a" => Some(1912),
		"18w46a" => Some(1910),
		"18w45a" => Some(1908),
		"18w44a" => Some(1907),
		"18w43c" => Some(1903),
		"18w43b" => Some(1902),
		"18w43a" => Some(1901),
		"1.13.2" => Some(1631),
		"1.13.2-pre2" => Some(1630),
		"1.13.2-pre1" => Some(1629),
		"1.13.1" => Some(1628),
		"1.13.1-pre2" => Some(1627),
		"1.13.1-pre1" => Some(1626),
		"18w33a" => Some(1625),
		"18w32a" => Some(1623),
		"18w31a" => Some(1622),
		"18w30b" => Some(1621),
		"18w30a" => Some(1620),
		"1.13" => Some(1519),
		"1.13-pre10" => Some(1518),
		"1.13-pre9" => Some(1517),
		"1.13-pre8" => Some(1516),
		"1.13-pre7" => Some(1513),
		"1.13-pre6" => Some(1512),
		"1.13-pre5" => Some(1511),
		"1.13-pre4" => Some(1504),
		"1.13-pre3" => Some(1503),
		"1.13-pre2" => Some(1502),
		"1.13-pre1" => Some(1501),
		"18w22c" => Some(1499),
		"18w22b" => Some(1498),
		"18w22a" => Some(1497),
		"18w21b" => Some(1496),
		"18w21a" => Some(1495),
		"18w20c" => Some(1493),
		"18w20b" => Some(1491),
		"18w20a" => Some(1489),
		"18w19b" => Some(1485),
		"18w19a" => Some(1484),
		"18w16a" => Some(1483),
		"18w15a" => Some(1482),
		"18w14b" => Some(1481),
		"18w14a" => Some(1479),
		"18w11a" => Some(1478),
		"18w10d" => Some(1477),
		"18w10c" => Some(1476),
		"18w10b" => Some(1474),
		"18w10a" => Some(1473),
		"18w09a" => Some(1472),
		"18w08b" => Some(1471),
		"18w08a" => Some(1470),
		"18w07c" => Some(1469),
		"18w07b" => Some(1468),
		"18w07a" => Some(1467),
		"18w06a" => Some(1466),
		"18w05a" => Some(1464),
		"18w03b" => Some(1463),
		"18w03a" => Some(1462),
		"18w02a" => Some(1461),
		"18w01a" => Some(1459),
		"17w50a" => Some(1457),
		"17w49b" => Some(1455),
		"17w49a" => Some(1454),
		"17w48a" => Some(1453),
		"17w47b" => Some(1452),
		"17w47a" => Some(1451),
		"17w46a" => Some(1449),
		"17w45b" => Some(1448),
		"17w45a" => Some(1447),
		"17w43b" => Some(1445),
		"17w43a" => Some(1444),
		"1.12.2" => Some(1343),
		"1.12.2-pre2" => Some(1342),
		"1.12.2-pre1" => Some(1341),
		"1.12.1" => Some(1241),
		"1.12.1-pre1" => Some(1240),
		"17w31a" => Some(1239),
		"1.12" => Some(1139),
		"1.12-pre7" => Some(1138),
		"1.12-pre6" => Some(1137),
		"1.12-pre5" => Some(1136),
		"1.12-pre4" => Some(1135),
		"1.12-pre3" => Some(1134),
		"1.12-pre2" => Some(1133),
		"1.12-pre1" => Some(1132),
		"17w18b" => Some(1131),
		"17w18a" => Some(1130),
		"17w17b" => Some(1129),
		"17w17a" => Some(1128),
		"17w16b" => Some(1127),
		"17w16a" => Some(1126),
		"17w15a" => Some(1125),
		"17w14a" => Some(1124),
		"17w13b" => Some(1123),
		"17w13a" => Some(1122),
		"17w06a" => Some(1022),
		"1.11.2" => Some(922),
		"1.11.1" => Some(921),
		"16w50a" => Some(920),
		"1.11" => Some(819),
		"1.11-pre1" => Some(818),
		"16w44a" => Some(817),
		"16w43a" => Some(816),
		"16w42a" => Some(815),
		"16w41a" => Some(814),
		"16w40a" => Some(813),
		"16w39c" => Some(812),
		"16w39b" => Some(811),
		"16w39a" => Some(809),
		"16w38a" => Some(807),
		"16w36a" => Some(805),
		"16w35a" => Some(803),
		"16w33a" => Some(802),
		"16w32b" => Some(801),
		"16w32a" => Some(800),
		"1.10.2" => Some(512),
		"1.10.1" => Some(511),
		"1.10" => Some(510),
		"1.10-pre2" => Some(507),
		"1.10-pre1" => Some(506),
		"16w21b" => Some(504),
		"16w21a" => Some(503),
		"16w20a" => Some(501),
		"1.9.4" => Some(184),
		"1.9.3" => Some(183),
		"1.9.3-pre3" => Some(182),
		"1.9.3-pre2" => Some(181),
		"1.9.3-pre1" => Some(180),
		"16w15b" => Some(179),
		"16w15a" => Some(178),
		"16w14a" => Some(177),
		"1.9.2" => Some(176),
		"1.9.1" => Some(175),
		"1.9.1-pre3" => Some(172),
		"1.9.1-pre2" => Some(171),
		"1.9.1-pre1" => Some(170),
		"1.9" => Some(169),
		"1.9-pre4" => Some(168),
		"1.9-pre3" => Some(167),
		"1.9-pre2" => Some(165),
		"1.9-pre1" => Some(164),
		"16w07b" => Some(163),
		"16w07a" => Some(162),
		"16w06a" => Some(161),
		"16w05b" => Some(160),
		"16w05a" => Some(159),
		"16w04a" => Some(158),
		"16w03a" => Some(157),
		"16w02a" => Some(156),
		"15w51b" => Some(155),
		"15w51a" => Some(154),
		"15w50a" => Some(153),
		"15w49b" => Some(152),
		"15w49a" => Some(151),
		"15w47c" => Some(150),
		"15w47b" => Some(149),
		"15w47a" => Some(148),
		"15w46a" => Some(146),
		"15w45a" => Some(145),
		"15w44b" => Some(143),
		"15w44a" => Some(142),
		"15w43c" => Some(141),
		"15w43b" => Some(140),
		"15w43a" => Some(139),
		"15w42a" => Some(138),
		"15w41b" => Some(137),
		"15w41a" => Some(136),
		"15w40b" => Some(134),
		"15w40a" => Some(133),
		"15w39c" => Some(132),
		"15w39b" => Some(131),
		"15w39a" => Some(130),
		"15w38b" => Some(129),
		"15w38a" => Some(128),
		"15w37a" => Some(127),
		"15w36d" => Some(126),
		"15w36c" => Some(125),
		"15w36b" => Some(124),
		"15w36a" => Some(123),
		"15w35e" => Some(122),
		"15w35d" => Some(121),
		"15w35c" => Some(120),
		"15w35b" => Some(119),
		"15w35a" => Some(118),
		"15w34d" => Some(117),
		"15w34c" => Some(116),
		"15w34b" => Some(115),
		"15w34a" => Some(114),
		"15w33c" => Some(112),
		"15w33b" => Some(111),
		"15w33a" => Some(111),
		"15w32c" => Some(104),
		"15w32b" => Some(103),
		"15w32a" => Some(100),
		_ => None,
	}
}
