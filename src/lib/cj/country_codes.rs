pub fn get_iso_code_3_from_iso_code_2(iso2: &str) -> &str {
    let result = COUNTRY_CODES
        .iter()
        .find(|&x| x.country_code_2 == iso2.to_uppercase());
    match result {
        Some(country_code_data) => country_code_data.country_code_3,
        None => "N/A",
    }
}

struct CountryCodeData {
    _country_name: &'static str,
    pub country_code_2: &'static str,
    pub country_code_3: &'static str,
}

/*
 * The following data was taken from the iso code list and processed as documented here:
 * https://gist.github.com/birdsarah/db905ead0fd3e8301fe8184867398c88
 */

const COUNTRY_CODES: [CountryCodeData; 209] = [
    CountryCodeData {
        _country_name: "Central African Republic (the)",
        country_code_2: "CF",
        country_code_3: "CAF",
    },
    CountryCodeData {
        _country_name: "Congo (the)",
        country_code_2: "CG",
        country_code_3: "COG",
    },
    CountryCodeData {
        _country_name: "Switzerland",
        country_code_2: "CH",
        country_code_3: "CHE",
    },
    CountryCodeData {
        _country_name: "Cote d'Ivoire",
        country_code_2: "CI",
        country_code_3: "CIV",
    },
    CountryCodeData {
        _country_name: "Cook Islands (the)",
        country_code_2: "CK",
        country_code_3: "COK",
    },
    CountryCodeData {
        _country_name: "Chile",
        country_code_2: "CL",
        country_code_3: "CHL",
    },
    CountryCodeData {
        _country_name: "Cameroon",
        country_code_2: "CM",
        country_code_3: "CMR",
    },
    CountryCodeData {
        _country_name: "China",
        country_code_2: "CN",
        country_code_3: "CHN",
    },
    CountryCodeData {
        _country_name: "Colombia",
        country_code_2: "CO",
        country_code_3: "COL",
    },
    CountryCodeData {
        _country_name: "Costa Rica",
        country_code_2: "CR",
        country_code_3: "CRI",
    },
    CountryCodeData {
        _country_name: "Cuba",
        country_code_2: "CU",
        country_code_3: "CUB",
    },
    CountryCodeData {
        _country_name: "Cabo Verde",
        country_code_2: "CV",
        country_code_3: "CPV",
    },
    CountryCodeData {
        _country_name: "Curacao",
        country_code_2: "CW",
        country_code_3: "CUW",
    },
    CountryCodeData {
        _country_name: "Christmas Island",
        country_code_2: "CX",
        country_code_3: "CXR",
    },
    CountryCodeData {
        _country_name: "Cyprus",
        country_code_2: "CY",
        country_code_3: "CYP",
    },
    CountryCodeData {
        _country_name: "Czechia",
        country_code_2: "CZ",
        country_code_3: "CZE",
    },
    CountryCodeData {
        _country_name: "Germany",
        country_code_2: "DE",
        country_code_3: "DEU",
    },
    CountryCodeData {
        _country_name: "Djibouti",
        country_code_2: "DJ",
        country_code_3: "DJI",
    },
    CountryCodeData {
        _country_name: "Denmark",
        country_code_2: "DK",
        country_code_3: "DNK",
    },
    CountryCodeData {
        _country_name: "Dominica",
        country_code_2: "DM",
        country_code_3: "DMA",
    },
    CountryCodeData {
        _country_name: "Dominican Republic (the)",
        country_code_2: "DO",
        country_code_3: "DOM",
    },
    CountryCodeData {
        _country_name: "Algeria",
        country_code_2: "DZ",
        country_code_3: "DZA",
    },
    CountryCodeData {
        _country_name: "Ecuador",
        country_code_2: "EC",
        country_code_3: "ECU",
    },
    CountryCodeData {
        _country_name: "Estonia",
        country_code_2: "EE",
        country_code_3: "EST",
    },
    CountryCodeData {
        _country_name: "Egypt",
        country_code_2: "EG",
        country_code_3: "EGY",
    },
    CountryCodeData {
        _country_name: "Western Sahara*",
        country_code_2: "EH",
        country_code_3: "ESH",
    },
    CountryCodeData {
        _country_name: "Eritrea",
        country_code_2: "ER",
        country_code_3: "ERI",
    },
    CountryCodeData {
        _country_name: "Spain",
        country_code_2: "ES",
        country_code_3: "ESP",
    },
    CountryCodeData {
        _country_name: "Ethiopia",
        country_code_2: "ET",
        country_code_3: "ETH",
    },
    CountryCodeData {
        _country_name: "Finland",
        country_code_2: "FI",
        country_code_3: "FIN",
    },
    CountryCodeData {
        _country_name: "Fiji",
        country_code_2: "FJ",
        country_code_3: "FJI",
    },
    CountryCodeData {
        _country_name: "Falkland Islands (the) [Malvinas]",
        country_code_2: "FK",
        country_code_3: "FLK",
    },
    CountryCodeData {
        _country_name: "Micronesia (Federated States of)",
        country_code_2: "FM",
        country_code_3: "FSM",
    },
    CountryCodeData {
        _country_name: "Faroe Islands (the)",
        country_code_2: "FO",
        country_code_3: "FRO",
    },
    CountryCodeData {
        _country_name: "France",
        country_code_2: "FR",
        country_code_3: "FRA",
    },
    CountryCodeData {
        _country_name: "Gabon",
        country_code_2: "GA",
        country_code_3: "GAB",
    },
    CountryCodeData {
        _country_name: "United Kingdom of Great Britain and Northern Ireland (the)",
        country_code_2: "GB",
        country_code_3: "GBR",
    },
    CountryCodeData {
        _country_name: "Grenada",
        country_code_2: "GD",
        country_code_3: "GRD",
    },
    CountryCodeData {
        _country_name: "Georgia",
        country_code_2: "GE",
        country_code_3: "GEO",
    },
    CountryCodeData {
        _country_name: "French Guiana",
        country_code_2: "GF",
        country_code_3: "GUF",
    },
    CountryCodeData {
        _country_name: "Guernsey",
        country_code_2: "GG",
        country_code_3: "GGY",
    },
    CountryCodeData {
        _country_name: "Ghana",
        country_code_2: "GH",
        country_code_3: "GHA",
    },
    CountryCodeData {
        _country_name: "Gibraltar",
        country_code_2: "GI",
        country_code_3: "GIB",
    },
    CountryCodeData {
        _country_name: "Greenland",
        country_code_2: "GL",
        country_code_3: "GRL",
    },
    CountryCodeData {
        _country_name: "Gambia (the)",
        country_code_2: "GM",
        country_code_3: "GMB",
    },
    CountryCodeData {
        _country_name: "Guinea",
        country_code_2: "GN",
        country_code_3: "GIN",
    },
    CountryCodeData {
        _country_name: "Guadeloupe",
        country_code_2: "GP",
        country_code_3: "GLP",
    },
    CountryCodeData {
        _country_name: "Equatorial Guinea",
        country_code_2: "GQ",
        country_code_3: "GNQ",
    },
    CountryCodeData {
        _country_name: "Greece",
        country_code_2: "GR",
        country_code_3: "GRC",
    },
    CountryCodeData {
        _country_name: "South Georgia and the South Sandwich Islands",
        country_code_2: "GS",
        country_code_3: "SGS",
    },
    CountryCodeData {
        _country_name: "Guatemala",
        country_code_2: "GT",
        country_code_3: "GTM",
    },
    CountryCodeData {
        _country_name: "Guam",
        country_code_2: "GU",
        country_code_3: "GUM",
    },
    CountryCodeData {
        _country_name: "Guinea-Bissau",
        country_code_2: "GW",
        country_code_3: "GNB",
    },
    CountryCodeData {
        _country_name: "Guyana",
        country_code_2: "GY",
        country_code_3: "GUY",
    },
    CountryCodeData {
        _country_name: "Hong Kong",
        country_code_2: "HK",
        country_code_3: "HKG",
    },
    CountryCodeData {
        _country_name: "Heard Island and McDonald Islands",
        country_code_2: "HM",
        country_code_3: "HMD",
    },
    CountryCodeData {
        _country_name: "Honduras",
        country_code_2: "HN",
        country_code_3: "HND",
    },
    CountryCodeData {
        _country_name: "Croatia",
        country_code_2: "HR",
        country_code_3: "HRV",
    },
    CountryCodeData {
        _country_name: "Haiti",
        country_code_2: "HT",
        country_code_3: "HTI",
    },
    CountryCodeData {
        _country_name: "Hungary",
        country_code_2: "HU",
        country_code_3: "HUN",
    },
    CountryCodeData {
        _country_name: "Indonesia",
        country_code_2: "ID",
        country_code_3: "IDN",
    },
    CountryCodeData {
        _country_name: "Ireland",
        country_code_2: "IE",
        country_code_3: "IRL",
    },
    CountryCodeData {
        _country_name: "Israel",
        country_code_2: "IL",
        country_code_3: "ISR",
    },
    CountryCodeData {
        _country_name: "Isle of Man",
        country_code_2: "IM",
        country_code_3: "IMN",
    },
    CountryCodeData {
        _country_name: "India",
        country_code_2: "IN",
        country_code_3: "IND",
    },
    CountryCodeData {
        _country_name: "British Indian Ocean Territory (the)",
        country_code_2: "IO",
        country_code_3: "IOT",
    },
    CountryCodeData {
        _country_name: "Iraq",
        country_code_2: "IQ",
        country_code_3: "IRQ",
    },
    CountryCodeData {
        _country_name: "Iran (Islamic Republic of)",
        country_code_2: "IR",
        country_code_3: "IRN",
    },
    CountryCodeData {
        _country_name: "Iceland",
        country_code_2: "IS",
        country_code_3: "ISL",
    },
    CountryCodeData {
        _country_name: "Italy",
        country_code_2: "IT",
        country_code_3: "ITA",
    },
    CountryCodeData {
        _country_name: "Jersey",
        country_code_2: "JE",
        country_code_3: "JEY",
    },
    CountryCodeData {
        _country_name: "Jamaica",
        country_code_2: "JM",
        country_code_3: "JAM",
    },
    CountryCodeData {
        _country_name: "Jordan",
        country_code_2: "JO",
        country_code_3: "JOR",
    },
    CountryCodeData {
        _country_name: "Japan",
        country_code_2: "JP",
        country_code_3: "JPN",
    },
    CountryCodeData {
        _country_name: "Kenya",
        country_code_2: "KE",
        country_code_3: "KEN",
    },
    CountryCodeData {
        _country_name: "Kyrgyzstan",
        country_code_2: "KG",
        country_code_3: "KGZ",
    },
    CountryCodeData {
        _country_name: "Cambodia",
        country_code_2: "KH",
        country_code_3: "KHM",
    },
    CountryCodeData {
        _country_name: "Kiribati",
        country_code_2: "KI",
        country_code_3: "KIR",
    },
    CountryCodeData {
        _country_name: "Comoros (the)",
        country_code_2: "KM",
        country_code_3: "COM",
    },
    CountryCodeData {
        _country_name: "Saint Kitts and Nevis",
        country_code_2: "KN",
        country_code_3: "KNA",
    },
    CountryCodeData {
        _country_name: "Korea (the Democratic People's Republic of)",
        country_code_2: "KP",
        country_code_3: "PRK",
    },
    CountryCodeData {
        _country_name: "Korea (the Republic of)",
        country_code_2: "KR",
        country_code_3: "KOR",
    },
    CountryCodeData {
        _country_name: "Kuwait",
        country_code_2: "KW",
        country_code_3: "KWT",
    },
    CountryCodeData {
        _country_name: "Cayman Islands (the)",
        country_code_2: "KY",
        country_code_3: "CYM",
    },
    CountryCodeData {
        _country_name: "Kazakhstan",
        country_code_2: "KZ",
        country_code_3: "KAZ",
    },
    CountryCodeData {
        _country_name: "Lao People's Democratic Republic (the)",
        country_code_2: "LA",
        country_code_3: "LAO",
    },
    CountryCodeData {
        _country_name: "Lebanon",
        country_code_2: "LB",
        country_code_3: "LBN",
    },
    CountryCodeData {
        _country_name: "Saint Lucia",
        country_code_2: "LC",
        country_code_3: "LCA",
    },
    CountryCodeData {
        _country_name: "Liechtenstein",
        country_code_2: "LI",
        country_code_3: "LIE",
    },
    CountryCodeData {
        _country_name: "Sri Lanka",
        country_code_2: "LK",
        country_code_3: "LKA",
    },
    CountryCodeData {
        _country_name: "Liberia",
        country_code_2: "LR",
        country_code_3: "LBR",
    },
    CountryCodeData {
        _country_name: "Lesotho",
        country_code_2: "LS",
        country_code_3: "LSO",
    },
    CountryCodeData {
        _country_name: "Lithuania",
        country_code_2: "LT",
        country_code_3: "LTU",
    },
    CountryCodeData {
        _country_name: "Luxembourg",
        country_code_2: "LU",
        country_code_3: "LUX",
    },
    CountryCodeData {
        _country_name: "Latvia",
        country_code_2: "LV",
        country_code_3: "LVA",
    },
    CountryCodeData {
        _country_name: "Libya",
        country_code_2: "LY",
        country_code_3: "LBY",
    },
    CountryCodeData {
        _country_name: "Morocco",
        country_code_2: "MA",
        country_code_3: "MAR",
    },
    CountryCodeData {
        _country_name: "Monaco",
        country_code_2: "MC",
        country_code_3: "MCO",
    },
    CountryCodeData {
        _country_name: "Moldova (the Republic of)",
        country_code_2: "MD",
        country_code_3: "MDA",
    },
    CountryCodeData {
        _country_name: "Montenegro",
        country_code_2: "ME",
        country_code_3: "MNE",
    },
    CountryCodeData {
        _country_name: "Saint Martin (French part)",
        country_code_2: "MF",
        country_code_3: "MAF",
    },
    CountryCodeData {
        _country_name: "Madagascar",
        country_code_2: "MG",
        country_code_3: "MDG",
    },
    CountryCodeData {
        _country_name: "Marshall Islands (the)",
        country_code_2: "MH",
        country_code_3: "MHL",
    },
    CountryCodeData {
        _country_name: "North Macedonia",
        country_code_2: "MK",
        country_code_3: "MKD",
    },
    CountryCodeData {
        _country_name: "Mali",
        country_code_2: "ML",
        country_code_3: "MLI",
    },
    CountryCodeData {
        _country_name: "Myanmar",
        country_code_2: "MM",
        country_code_3: "MMR",
    },
    CountryCodeData {
        _country_name: "Mongolia",
        country_code_2: "MN",
        country_code_3: "MNG",
    },
    CountryCodeData {
        _country_name: "Macao",
        country_code_2: "MO",
        country_code_3: "MAC",
    },
    CountryCodeData {
        _country_name: "Northern Mariana Islands (the)",
        country_code_2: "MP",
        country_code_3: "MNP",
    },
    CountryCodeData {
        _country_name: "Martinique",
        country_code_2: "MQ",
        country_code_3: "MTQ",
    },
    CountryCodeData {
        _country_name: "Mauritania",
        country_code_2: "MR",
        country_code_3: "MRT",
    },
    CountryCodeData {
        _country_name: "Montserrat",
        country_code_2: "MS",
        country_code_3: "MSR",
    },
    CountryCodeData {
        _country_name: "Malta",
        country_code_2: "MT",
        country_code_3: "MLT",
    },
    CountryCodeData {
        _country_name: "Mauritius",
        country_code_2: "MU",
        country_code_3: "MUS",
    },
    CountryCodeData {
        _country_name: "Maldives",
        country_code_2: "MV",
        country_code_3: "MDV",
    },
    CountryCodeData {
        _country_name: "Malawi",
        country_code_2: "MW",
        country_code_3: "MWI",
    },
    CountryCodeData {
        _country_name: "Mexico",
        country_code_2: "MX",
        country_code_3: "MEX",
    },
    CountryCodeData {
        _country_name: "Malaysia",
        country_code_2: "MY",
        country_code_3: "MYS",
    },
    CountryCodeData {
        _country_name: "Mozambique",
        country_code_2: "MZ",
        country_code_3: "MOZ",
    },
    CountryCodeData {
        _country_name: "Namibia",
        country_code_2: "NA",
        country_code_3: "NAM",
    },
    CountryCodeData {
        _country_name: "New Caledonia",
        country_code_2: "NC",
        country_code_3: "NCL",
    },
    CountryCodeData {
        _country_name: "Niger (the)",
        country_code_2: "NE",
        country_code_3: "NER",
    },
    CountryCodeData {
        _country_name: "Norfolk Island",
        country_code_2: "NF",
        country_code_3: "NFK",
    },
    CountryCodeData {
        _country_name: "Nigeria",
        country_code_2: "NG",
        country_code_3: "NGA",
    },
    CountryCodeData {
        _country_name: "Nicaragua",
        country_code_2: "NI",
        country_code_3: "NIC",
    },
    CountryCodeData {
        _country_name: "Netherlands (the)",
        country_code_2: "NL",
        country_code_3: "NLD",
    },
    CountryCodeData {
        _country_name: "Norway",
        country_code_2: "NO",
        country_code_3: "NOR",
    },
    CountryCodeData {
        _country_name: "Nepal",
        country_code_2: "NP",
        country_code_3: "NPL",
    },
    CountryCodeData {
        _country_name: "Nauru",
        country_code_2: "NR",
        country_code_3: "NRU",
    },
    CountryCodeData {
        _country_name: "Niue",
        country_code_2: "NU",
        country_code_3: "NIU",
    },
    CountryCodeData {
        _country_name: "New Zealand",
        country_code_2: "NZ",
        country_code_3: "NZL",
    },
    CountryCodeData {
        _country_name: "Oman",
        country_code_2: "OM",
        country_code_3: "OMN",
    },
    CountryCodeData {
        _country_name: "Panama",
        country_code_2: "PA",
        country_code_3: "PAN",
    },
    CountryCodeData {
        _country_name: "Peru",
        country_code_2: "PE",
        country_code_3: "PER",
    },
    CountryCodeData {
        _country_name: "French Polynesia",
        country_code_2: "PF",
        country_code_3: "PYF",
    },
    CountryCodeData {
        _country_name: "Papua New Guinea",
        country_code_2: "PG",
        country_code_3: "PNG",
    },
    CountryCodeData {
        _country_name: "Philippines (the)",
        country_code_2: "PH",
        country_code_3: "PHL",
    },
    CountryCodeData {
        _country_name: "Pakistan",
        country_code_2: "PK",
        country_code_3: "PAK",
    },
    CountryCodeData {
        _country_name: "Poland",
        country_code_2: "PL",
        country_code_3: "POL",
    },
    CountryCodeData {
        _country_name: "Saint Pierre and Miquelon",
        country_code_2: "PM",
        country_code_3: "SPM",
    },
    CountryCodeData {
        _country_name: "Pitcairn",
        country_code_2: "PN",
        country_code_3: "PCN",
    },
    CountryCodeData {
        _country_name: "Puerto Rico",
        country_code_2: "PR",
        country_code_3: "PRI",
    },
    CountryCodeData {
        _country_name: "Palestine, State of",
        country_code_2: "PS",
        country_code_3: "PSE",
    },
    CountryCodeData {
        _country_name: "Portugal",
        country_code_2: "PT",
        country_code_3: "PRT",
    },
    CountryCodeData {
        _country_name: "Palau",
        country_code_2: "PW",
        country_code_3: "PLW",
    },
    CountryCodeData {
        _country_name: "Paraguay",
        country_code_2: "PY",
        country_code_3: "PRY",
    },
    CountryCodeData {
        _country_name: "Qatar",
        country_code_2: "QA",
        country_code_3: "QAT",
    },
    CountryCodeData {
        _country_name: "Reunion",
        country_code_2: "RE",
        country_code_3: "REU",
    },
    CountryCodeData {
        _country_name: "Romania",
        country_code_2: "RO",
        country_code_3: "ROU",
    },
    CountryCodeData {
        _country_name: "Serbia",
        country_code_2: "RS",
        country_code_3: "SRB",
    },
    CountryCodeData {
        _country_name: "Russian Federation (the)",
        country_code_2: "RU",
        country_code_3: "RUS",
    },
    CountryCodeData {
        _country_name: "Rwanda",
        country_code_2: "RW",
        country_code_3: "RWA",
    },
    CountryCodeData {
        _country_name: "Saudi Arabia",
        country_code_2: "SA",
        country_code_3: "SAU",
    },
    CountryCodeData {
        _country_name: "Solomon Islands",
        country_code_2: "SB",
        country_code_3: "SLB",
    },
    CountryCodeData {
        _country_name: "Seychelles",
        country_code_2: "SC",
        country_code_3: "SYC",
    },
    CountryCodeData {
        _country_name: "Sudan (the)",
        country_code_2: "SD",
        country_code_3: "SDN",
    },
    CountryCodeData {
        _country_name: "Sweden",
        country_code_2: "SE",
        country_code_3: "SWE",
    },
    CountryCodeData {
        _country_name: "Singapore",
        country_code_2: "SG",
        country_code_3: "SGP",
    },
    CountryCodeData {
        _country_name: "Saint Helena, Ascension and Tristan da Cunha",
        country_code_2: "SH",
        country_code_3: "SHN",
    },
    CountryCodeData {
        _country_name: "Slovenia",
        country_code_2: "SI",
        country_code_3: "SVN",
    },
    CountryCodeData {
        _country_name: "Svalbard and Jan Mayen",
        country_code_2: "SJ",
        country_code_3: "SJM",
    },
    CountryCodeData {
        _country_name: "Slovakia",
        country_code_2: "SK",
        country_code_3: "SVK",
    },
    CountryCodeData {
        _country_name: "Sierra Leone",
        country_code_2: "SL",
        country_code_3: "SLE",
    },
    CountryCodeData {
        _country_name: "San Marino",
        country_code_2: "SM",
        country_code_3: "SMR",
    },
    CountryCodeData {
        _country_name: "Senegal",
        country_code_2: "SN",
        country_code_3: "SEN",
    },
    CountryCodeData {
        _country_name: "Somalia",
        country_code_2: "SO",
        country_code_3: "SOM",
    },
    CountryCodeData {
        _country_name: "Suriname",
        country_code_2: "SR",
        country_code_3: "SUR",
    },
    CountryCodeData {
        _country_name: "South Sudan",
        country_code_2: "SS",
        country_code_3: "SSD",
    },
    CountryCodeData {
        _country_name: "Sao Tome and Principe",
        country_code_2: "ST",
        country_code_3: "STP",
    },
    CountryCodeData {
        _country_name: "El Salvador",
        country_code_2: "SV",
        country_code_3: "SLV",
    },
    CountryCodeData {
        _country_name: "Sint Maarten (Dutch part)",
        country_code_2: "SX",
        country_code_3: "SXM",
    },
    CountryCodeData {
        _country_name: "Syrian Arab Republic (the)",
        country_code_2: "SY",
        country_code_3: "SYR",
    },
    CountryCodeData {
        _country_name: "Eswatini",
        country_code_2: "SZ",
        country_code_3: "SWZ",
    },
    CountryCodeData {
        _country_name: "Turks and Caicos Islands (the)",
        country_code_2: "TC",
        country_code_3: "TCA",
    },
    CountryCodeData {
        _country_name: "Chad",
        country_code_2: "TD",
        country_code_3: "TCD",
    },
    CountryCodeData {
        _country_name: "French Southern Territories (the)",
        country_code_2: "TF",
        country_code_3: "ATF",
    },
    CountryCodeData {
        _country_name: "Togo",
        country_code_2: "TG",
        country_code_3: "TGO",
    },
    CountryCodeData {
        _country_name: "Thailand",
        country_code_2: "TH",
        country_code_3: "THA",
    },
    CountryCodeData {
        _country_name: "Tajikistan",
        country_code_2: "TJ",
        country_code_3: "TJK",
    },
    CountryCodeData {
        _country_name: "Tokelau",
        country_code_2: "TK",
        country_code_3: "TKL",
    },
    CountryCodeData {
        _country_name: "Timor-Leste",
        country_code_2: "TL",
        country_code_3: "TLS",
    },
    CountryCodeData {
        _country_name: "Turkmenistan",
        country_code_2: "TM",
        country_code_3: "TKM",
    },
    CountryCodeData {
        _country_name: "Tunisia",
        country_code_2: "TN",
        country_code_3: "TUN",
    },
    CountryCodeData {
        _country_name: "Tonga",
        country_code_2: "TO",
        country_code_3: "TON",
    },
    CountryCodeData {
        _country_name: "Turkey",
        country_code_2: "TR",
        country_code_3: "TUR",
    },
    CountryCodeData {
        _country_name: "Trinidad and Tobago",
        country_code_2: "TT",
        country_code_3: "TTO",
    },
    CountryCodeData {
        _country_name: "Tuvalu",
        country_code_2: "TV",
        country_code_3: "TUV",
    },
    CountryCodeData {
        _country_name: "Taiwan (Province of China)",
        country_code_2: "TW",
        country_code_3: "TWN",
    },
    CountryCodeData {
        _country_name: "Tanzania, the United Republic of",
        country_code_2: "TZ",
        country_code_3: "TZA",
    },
    CountryCodeData {
        _country_name: "Ukraine",
        country_code_2: "UA",
        country_code_3: "UKR",
    },
    CountryCodeData {
        _country_name: "Uganda",
        country_code_2: "UG",
        country_code_3: "UGA",
    },
    CountryCodeData {
        _country_name: "United States Minor Outlying Islands (the)",
        country_code_2: "UM",
        country_code_3: "UMI",
    },
    CountryCodeData {
        _country_name: "United States of America (the)",
        country_code_2: "US",
        country_code_3: "USA",
    },
    CountryCodeData {
        _country_name: "Uruguay",
        country_code_2: "UY",
        country_code_3: "URY",
    },
    CountryCodeData {
        _country_name: "Uzbekistan",
        country_code_2: "UZ",
        country_code_3: "UZB",
    },
    CountryCodeData {
        _country_name: "Holy See (the)",
        country_code_2: "VA",
        country_code_3: "VAT",
    },
    CountryCodeData {
        _country_name: "Saint Vincent and the Grenadines",
        country_code_2: "VC",
        country_code_3: "VCT",
    },
    CountryCodeData {
        _country_name: "Venezuela (Bolivarian Republic of)",
        country_code_2: "VE",
        country_code_3: "VEN",
    },
    CountryCodeData {
        _country_name: "Virgin Islands (British)",
        country_code_2: "VG",
        country_code_3: "VGB",
    },
    CountryCodeData {
        _country_name: "Virgin Islands (U.S.)",
        country_code_2: "VI",
        country_code_3: "VIR",
    },
    CountryCodeData {
        _country_name: "Viet Nam",
        country_code_2: "VN",
        country_code_3: "VNM",
    },
    CountryCodeData {
        _country_name: "Vanuatu",
        country_code_2: "VU",
        country_code_3: "VUT",
    },
    CountryCodeData {
        _country_name: "Wallis and Futuna",
        country_code_2: "WF",
        country_code_3: "WLF",
    },
    CountryCodeData {
        _country_name: "Samoa",
        country_code_2: "WS",
        country_code_3: "WSM",
    },
    CountryCodeData {
        _country_name: "Yemen",
        country_code_2: "YE",
        country_code_3: "YEM",
    },
    CountryCodeData {
        _country_name: "Mayotte",
        country_code_2: "YT",
        country_code_3: "MYT",
    },
    CountryCodeData {
        _country_name: "South Africa",
        country_code_2: "ZA",
        country_code_3: "ZAF",
    },
    CountryCodeData {
        _country_name: "Zambia",
        country_code_2: "ZM",
        country_code_3: "ZMB",
    },
    CountryCodeData {
        _country_name: "Zimbabwe",
        country_code_2: "ZW",
        country_code_3: "ZWE",
    },
];

#[cfg(test)]
mod test_country_codes {
    use super::*;

    #[test]
    fn returns_three_from_two_easy() {
        assert_eq!(get_iso_code_3_from_iso_code_2("us"), "USA");
    }

    #[test]
    fn returns_three_from_two_handles_case() {
        assert_eq!(get_iso_code_3_from_iso_code_2("cG"), "COG");
    }

    #[test]
    fn returns_na_if_not_found() {
        assert_eq!(get_iso_code_3_from_iso_code_2("gfd"), "N/A");
        assert_eq!(get_iso_code_3_from_iso_code_2(""), "N/A");
    }
}
