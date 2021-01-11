use serde::Serialize;

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub(crate) enum Weapon {
    Unknown,
    AirStrike,
    AliBabasWeeBooties,
    Ambassador,
    Amputator,
    ApocoFists,
    ApSap,
    Atomizer,
    AWPerHand,
    Axtinguisher,
    BabyFaceBlaster,
    Backburner,
    BackScatter,
    BackScratcher,
    BASEJumper,
    Bat,
    BatOuttaHell,
    BatSaber,
    BattalionsBackup,
    BazaarBargain,
    BeggarsBazooka,
    BigEarner,
    BigKill,
    BlackBox,
    BlackRose,
    Blutsauger,
    Bonesaw,
    BonkAtomicPunch,
    Bootlegger,
    BostonBasher,
    Bottle,
    BrassBeast,
    BreadBite,
    BuffaloSteakSandvich,
    BuffBanner,
    Bushwacka,
    CandyCane,
    CAPPER,
    CharginTarge,
    ClaidheamhMor,
    Classic,
    CleanersCarbine,
    CloakAndDagger,
    Concheror,
    ConniversKunai,
    ConscientiousObjector,
    ConstructionPDA,
    CowMangler5000,
    CozyCamper,
    CrossingGuard,
    CrusadersCrossbow,
    DalokohsBar,
    Deflector,
    Degreaser,
    DarwinsDangerShield,
    DeadRinger,
    DestructionPDA,
    Detonator,
    Diamondback,
    DisguiseKitPDA,
    DirectHit,
    DisciplinaryAction,
    DragonsFury,
    Enforcer,
    EnthusiastsTimepiece,
    Equalizer,
    EscapePlan,
    EurekaEffect,
    EvictionNotice,
    Eyelander,
    FamilyBusiness,
    FanOWar,
    FireAxe,
    Fishcake,
    Fists,
    FistsOfSteel,
    FlameThrower,
    FlareGun,
    FlyingGuillotine,
    ForceANature,
    FortifiedCompound,
    FreedomStaff,
    FrontierJustice,
    FryingPan,
    GasPasser,
    GigarCounter,
    GlovesOfRunningUrgently,
    GoldenFryingPan,
    GoldenWrench,
    GrenadeLauncher,
    Gunboats,
    Gunslinger,
    HalfZatoichi,
    HamShank,
    HitmansHeatmaker,
    HolidayPunch,
    HolyMackerel,
    Homewrecker,
    Huntsman,
    HorselessHeadlessHorsemannsHeadtaker,
    HotHand,
    HuoLongHeater,
    InvisWatch,
    IronBomber,
    IronCurtain,
    Jag,
    Jarate,
    KillingGlovesOfBoxing,
    Knife,
    Kritzkrieg,
    Kukri,
    LEtranger,
    LibertyLauncher,
    LochNLoad,
    Lollichop,
    LooseCannon,
    Lugermorph,
    Machina,
    Manmelter,
    Mantreads,
    MarketGardener,
    Maul,
    MediGun,
    MemoryMaker,
    Minigun,
    MutatedMilk,
    Natascha,
    NecroSmasher,
    NeonAnnihilator,
    NessiesNineIron,
    NostromoNapalmer,
    Original,
    Overdose,
    PainTrain,
    PanicAttack,
    PersianPersuader,
    Phlogistinator,
    Pistol,
    Pomson6000,
    PostalPummeler,
    Powerjack,
    PrettyBoysPocketPistol,
    PrinnyMachete,
    Quackenbirdt,
    QuickFix,
    QuickiebombLauncher,
    Rainblower,
    Razorback,
    RedTapeRecorder,
    RescueRanger,
    ReserveShooter,
    Revolver,
    RighteousBison,
    RoboSandvich,
    RocketJumper,
    RocketLauncher,
    Sandman,
    Sandvich,
    Sapper,
    Saxxy,
    Scattergun,
    ScorchShot,
    ScotsmansSkullcutter,
    ScottishHandshake,
    ScottishResistance,
    SecondBanana,
    SelfAwareBeautyMark,
    Shahanshah,
    SharpDresser,
    SharpenedVolcanoFragment,
    ShootingStar,
    ShortCircuit,
    Shortstop,
    Shotgun,
    Shovel,
    SMG,
    SnackAttack,
    SniperRifle,
    SodaPopper,
    SolemnVow,
    SouthernHospitality,
    SplendidScreen,
    Spycicle,
    StickybombLauncher,
    StickyJumper,
    SunOnAStick,
    SydneySleeper,
    SyringeGun,
    ThermalThruster,
    ThreeRuneBlade,
    ThirdDegree,
    TideTurner,
    Tomislav,
    Toolbox,
    TribalmansShiv,
    Ubersaw,
    UllapoolCaber,
    UnarmedCombat,
    Vaccinator,
    VitaSaw,
    WangaPrick,
    WarriorsSpirit,
    Widowmaker,
    Winger,
    Wrangler,
    WrapAssassin,
    Wrench,
    YourEternalReward,
}

// https://wiki.alliedmods.net/Team_fortress_2_item_definition_indexes#Weapons

pub(crate) fn index_to_weapon(index: i32) -> Weapon {
    match index {
        13 => Weapon::Scattergun,
        200 => Weapon::Scattergun, // Renamed/Strange
        45 => Weapon::ForceANature,
        220 => Weapon::Shortstop,
        448 => Weapon::SodaPopper,
        669 => Weapon::Scattergun, // Festive
        772 => Weapon::BabyFaceBlaster,
        799 => Weapon::Scattergun,    // Silver Botkiller Scattergun Mk.I
        808 => Weapon::Scattergun,    // Gold Botkiller Scattergun Mk.I
        888 => Weapon::Scattergun,    // Rust Botkiller Scattergun Mk.I
        897 => Weapon::Scattergun,    // Blood Botkiller Scattergun Mk.I
        906 => Weapon::Scattergun,    // Carbonado Botkiller Scattergun Mk.I
        915 => Weapon::Scattergun,    // Diamond Botkiller Scattergun Mk.I
        964 => Weapon::Scattergun,    // Silver Botkiller Scattergun Mk.II
        973 => Weapon::Scattergun,    // Gold Botkiller Scattergun Mk.II
        1078 => Weapon::ForceANature, // Festive
        1103 => Weapon::BackScatter,
        15002 => Weapon::Scattergun, // Night Terror
        15015 => Weapon::Scattergun, // Tartan Torpedo
        15021 => Weapon::Scattergun, // Country Crusher
        15029 => Weapon::Scattergun, // Backcountry Blaster
        15036 => Weapon::Scattergun, // Spruce Deuce
        15053 => Weapon::Scattergun, // Current Event
        15065 => Weapon::Scattergun, // Macabre Web
        15069 => Weapon::Scattergun, // Nutcracker
        15106 => Weapon::Scattergun, // Blue Mew
        15107 => Weapon::Scattergun, // Flower Power
        15108 => Weapon::Scattergun, // Shot to Hell
        15131 => Weapon::Scattergun, // Coffin Nail
        15151 => Weapon::Scattergun, // Killer Bee
        15157 => Weapon::Scattergun, // Corsair

        23 => Weapon::Pistol,
        209 => Weapon::Pistol, // Renamed/Strange
        46 => Weapon::BonkAtomicPunch,
        160 => Weapon::Lugermorph, // Vintage
        449 => Weapon::Winger,
        773 => Weapon::PrettyBoysPocketPistol,
        812 => Weapon::FlyingGuillotine,
        833 => Weapon::FlyingGuillotine, // Genuine
        1121 => Weapon::MutatedMilk,
        1145 => Weapon::BonkAtomicPunch, // Festive
        15013 => Weapon::Pistol,         // Red Rock Roscoe
        15018 => Weapon::Pistol,         // Homemade Heater
        15035 => Weapon::Pistol,         // Hickory Holepuncher
        15041 => Weapon::Pistol,         // Local Hero
        15046 => Weapon::Pistol,         // Black Dahlia
        15056 => Weapon::Pistol,         // Sandstone Special
        15060 => Weapon::Pistol,         // Macabre Web
        15061 => Weapon::Pistol,         // Nutcracker
        15100 => Weapon::Pistol,         // Blue Mew
        15101 => Weapon::Pistol,         // Brain Candy
        15102 => Weapon::Pistol,         // Shot to Hell
        15126 => Weapon::Pistol,         // Dressed To Kill
        15148 => Weapon::Pistol,         // Blitzkrieg
        30666 => Weapon::CAPPER,

        0 => Weapon::Bat,
        190 => Weapon::Bat, // Renamed/Strange
        44 => Weapon::Sandman,
        221 => Weapon::HolyMackerel,
        264 => Weapon::FryingPan,
        317 => Weapon::CandyCane,
        325 => Weapon::BostonBasher,
        349 => Weapon::SunOnAStick,
        355 => Weapon::FanOWar,
        423 => Weapon::Saxxy,
        450 => Weapon::Atomizer,
        452 => Weapon::ThreeRuneBlade,
        474 => Weapon::ConscientiousObjector,
        572 => Weapon::UnarmedCombat,
        648 => Weapon::WrapAssassin,
        660 => Weapon::Bat, // Festive
        880 => Weapon::FreedomStaff,
        939 => Weapon::BatOuttaHell,
        954 => Weapon::MemoryMaker,
        999 => Weapon::HolyMackerel, // Festive
        1013 => Weapon::HamShank,
        1071 => Weapon::GoldenFryingPan,
        1123 => Weapon::NecroSmasher,
        1127 => Weapon::CrossingGuard,
        30667 => Weapon::BatSaber,
        30758 => Weapon::PrinnyMachete,

        18 => Weapon::RocketLauncher,
        205 => Weapon::RocketLauncher, // Renamed/Strange
        127 => Weapon::DirectHit,
        228 => Weapon::BlackBox,
        237 => Weapon::RocketJumper,
        414 => Weapon::LibertyLauncher,
        441 => Weapon::CowMangler5000,
        513 => Weapon::Original,
        658 => Weapon::RocketLauncher, // Festive
        730 => Weapon::BeggarsBazooka,
        800 => Weapon::RocketLauncher, // Silver Botkiller Rocket Launcher Mk.I
        809 => Weapon::RocketLauncher, // Gold Botkiller Rocket Launcher Mk.I
        889 => Weapon::RocketLauncher, // Rust Botkiller Rocket Launcher Mk.I
        898 => Weapon::RocketLauncher, // Blood Botkiller Rocket Launcher Mk.I
        907 => Weapon::RocketLauncher, // Carbonado Botkiller Rocket Launcher Mk.I
        916 => Weapon::RocketLauncher, // Diamond Botkiller Rocket Launcher Mk.I
        965 => Weapon::RocketLauncher, // Silver Botkiller Rocket Launcher Mk.II
        974 => Weapon::RocketLauncher, // Gold Botkiller Rocket Launcher Mk.II
        1085 => Weapon::BlackBox,      // Festive
        1104 => Weapon::AirStrike,
        15006 => Weapon::RocketLauncher, // Woodland Warrior
        15014 => Weapon::RocketLauncher, // Sand Cannon
        15028 => Weapon::RocketLauncher, // American Pastoral
        15043 => Weapon::RocketLauncher, // Smalltown Bringdown
        15052 => Weapon::RocketLauncher, // Shell Shocker
        15057 => Weapon::RocketLauncher, // Aqua Marine
        15081 => Weapon::RocketLauncher, // Autumn
        15104 => Weapon::RocketLauncher, // Blue Mew
        15105 => Weapon::RocketLauncher, // Brain Candy
        15129 => Weapon::RocketLauncher, // Coffin Nail
        15130 => Weapon::RocketLauncher, // High Roller's
        15150 => Weapon::RocketLauncher, // Warhawk

        10 => Weapon::Shotgun,
        199 => Weapon::Shotgun, // Renamed/Strange
        129 => Weapon::BuffBanner,
        133 => Weapon::Gunboats,
        226 => Weapon::BattalionsBackup,
        354 => Weapon::Concheror,
        415 => Weapon::ReserveShooter,
        442 => Weapon::RighteousBison,
        444 => Weapon::Mantreads,
        1001 => Weapon::BuffBanner, // Festive
        1101 => Weapon::BASEJumper,
        1141 => Weapon::Shotgun, // Festive
        1153 => Weapon::PanicAttack,
        15003 => Weapon::Shotgun, // Backwoods Boomstick
        15016 => Weapon::Shotgun, // Rustic Ruiner
        15044 => Weapon::Shotgun, // Civic Duty
        15047 => Weapon::Shotgun, // Lightning Rod
        15085 => Weapon::Shotgun, // Autumn
        15109 => Weapon::Shotgun, // Flower Power
        15132 => Weapon::Shotgun, // Coffin Nail
        15133 => Weapon::Shotgun, // Dressed to Kill
        15152 => Weapon::Shotgun, // Red Bear

        6 => Weapon::Shovel,
        196 => Weapon::Shovel, // Renamed/Strange
        128 => Weapon::Equalizer,
        154 => Weapon::PainTrain,
        357 => Weapon::HalfZatoichi,
        416 => Weapon::MarketGardener,
        447 => Weapon::DisciplinaryAction,
        775 => Weapon::EscapePlan,

        21 => Weapon::FlameThrower,
        208 => Weapon::FlameThrower, // Renamed/Strange
        40 => Weapon::Backburner,
        215 => Weapon::Degreaser,
        594 => Weapon::Phlogistinator,
        659 => Weapon::FlameThrower, // Festive
        741 => Weapon::Rainblower,
        798 => Weapon::FlameThrower, // Silver Botkiller Flame Thrower Mk.I
        807 => Weapon::FlameThrower, // Gold Botkiller Flame Thrower Mk.I
        887 => Weapon::FlameThrower, // Rust Botkiller Flame Thrower Mk.I
        896 => Weapon::FlameThrower, // Blood Botkiller Flame Thrower Mk.I
        905 => Weapon::FlameThrower, // Carbonado Botkiller Flame Thrower Mk.I
        914 => Weapon::FlameThrower, // Diamond Botkiller Flame Thrower Mk.I
        963 => Weapon::FlameThrower, // Silver Botkiller Flame Thrower Mk.II
        972 => Weapon::FlameThrower, // Gold Botkiller Flame Thrower Mk.II
        1146 => Weapon::Backburner,  // Festive
        1178 => Weapon::DragonsFury,
        15005 => Weapon::FlameThrower, // Forest Fire
        15017 => Weapon::FlameThrower, // Barn Burner
        15030 => Weapon::FlameThrower, // Bovine Blazemaker
        15034 => Weapon::FlameThrower, // Earth, Sky and Fire
        15049 => Weapon::FlameThrower, // Flash Fryer
        15054 => Weapon::FlameThrower, // Turbine Torcher
        15066 => Weapon::FlameThrower, // Autumn
        15067 => Weapon::FlameThrower, // Pumpkin Patch
        15068 => Weapon::FlameThrower, // Nutcracker
        15089 => Weapon::FlameThrower, // Balloonicorn
        15090 => Weapon::FlameThrower, // Rainbow
        15115 => Weapon::FlameThrower, // Coffin Nai
        15141 => Weapon::FlameThrower, // Warhawk
        30474 => Weapon::NostromoNapalmer,

        12 => Weapon::Shotgun,
        39 => Weapon::FlareGun,
        351 => Weapon::Detonator,
        595 => Weapon::Manmelter,
        740 => Weapon::ScorchShot,
        1081 => Weapon::FlareGun, // Festive
        1179 => Weapon::ThermalThruster,
        1180 => Weapon::GasPasser,

        2 => Weapon::FireAxe,
        192 => Weapon::FireAxe, // Renamed/Strange
        38 => Weapon::Axtinguisher,
        153 => Weapon::Homewrecker,
        214 => Weapon::Powerjack,
        326 => Weapon::BackScratcher,
        348 => Weapon::SharpenedVolcanoFragment,
        457 => Weapon::PostalPummeler,
        466 => Weapon::Maul,
        593 => Weapon::ThirdDegree,
        739 => Weapon::Lollichop,
        813 => Weapon::NeonAnnihilator,
        834 => Weapon::NeonAnnihilator, // Genuine
        1000 => Weapon::Axtinguisher,   // Festive
        1181 => Weapon::HotHand,

        19 => Weapon::GrenadeLauncher,
        206 => Weapon::GrenadeLauncher, // Renamed/Strange
        308 => Weapon::LochNLoad,
        405 => Weapon::AliBabasWeeBooties,
        608 => Weapon::Bootlegger,
        996 => Weapon::LooseCannon,
        1007 => Weapon::GrenadeLauncher, // Festive
        1151 => Weapon::IronBomber,
        15077 => Weapon::GrenadeLauncher, // Autumn
        15079 => Weapon::GrenadeLauncher, // Macabre Web
        15091 => Weapon::GrenadeLauncher, // Rainbow
        15092 => Weapon::GrenadeLauncher, // Sweet Dreams
        15116 => Weapon::GrenadeLauncher, // Coffin Nail
        15117 => Weapon::GrenadeLauncher, // Top Shelf
        15142 => Weapon::GrenadeLauncher, // Warhawk
        15158 => Weapon::GrenadeLauncher, // Butcher Bird

        20 => Weapon::StickybombLauncher,
        207 => Weapon::StickybombLauncher, // Renamed/Strange
        130 => Weapon::ScottishResistance,
        131 => Weapon::CharginTarge,
        265 => Weapon::StickyJumper,
        406 => Weapon::SplendidScreen,
        661 => Weapon::StickybombLauncher, // Festive
        797 => Weapon::StickybombLauncher, // Silver Botkiller Stickybomb Launcher Mk.I
        806 => Weapon::StickybombLauncher, // Gold Botkiller Stickybomb Launcher Mk.I
        886 => Weapon::StickybombLauncher, // Rust Botkiller Stickybomb Launcher Mk.I
        895 => Weapon::StickybombLauncher, // Blood Botkiller Stickybomb Launcher Mk.I
        904 => Weapon::StickybombLauncher, // Carbonado Botkiller Stickybomb Launcher Mk.I
        913 => Weapon::StickybombLauncher, // Diamond Botkiller Stickybomb Launcher Mk.I
        962 => Weapon::StickybombLauncher, // Silver Botkiller Stickybomb Launcher Mk.II
        971 => Weapon::StickybombLauncher, // Gold Botkiller Stickybomb Launcher Mk.II
        1099 => Weapon::TideTurner,
        1144 => Weapon::CharginTarge, // Festive
        1150 => Weapon::QuickiebombLauncher,
        15009 => Weapon::StickybombLauncher, // Sudden Flurry
        15012 => Weapon::StickybombLauncher, // Carpet Bomber
        15024 => Weapon::StickybombLauncher, // Blasted Bombardier
        15038 => Weapon::StickybombLauncher, // Rooftop Wrangler
        15045 => Weapon::StickybombLauncher, // Liquid Asset
        15048 => Weapon::StickybombLauncher, // Pink Elephant
        15082 => Weapon::StickybombLauncher, // Autumn
        15083 => Weapon::StickybombLauncher, // Pumpkin Patch
        15084 => Weapon::StickybombLauncher, // Macabre Web
        15113 => Weapon::StickybombLauncher, // Sweet Dreams
        15137 => Weapon::StickybombLauncher, // Coffin Nail
        15138 => Weapon::StickybombLauncher, // Dressed to Kill
        15155 => Weapon::StickybombLauncher, // Blitzkrieg

        1 => Weapon::Bottle,
        191 => Weapon::Bottle, // Renamed/Strange
        132 => Weapon::Eyelander,
        172 => Weapon::ScotsmansSkullcutter,
        266 => Weapon::HorselessHeadlessHorsemannsHeadtaker,
        307 => Weapon::UllapoolCaber,
        327 => Weapon::ClaidheamhMor,
        404 => Weapon::PersianPersuader,
        482 => Weapon::NessiesNineIron,
        609 => Weapon::ScottishHandshake,
        1082 => Weapon::Eyelander, // Festive

        15 => Weapon::Minigun,
        202 => Weapon::Minigun, // Renamed/Strange
        41 => Weapon::Natascha,
        298 => Weapon::IronCurtain,
        312 => Weapon::BrassBeast,
        424 => Weapon::Tomislav,
        654 => Weapon::Minigun, // Festive
        793 => Weapon::Minigun, // Silver Botkiller Minigun Mk.I
        802 => Weapon::Minigun, // Gold Botkiller Minigun Mk.I
        811 => Weapon::HuoLongHeater,
        832 => Weapon::HuoLongHeater, // Genuine
        850 => Weapon::Deflector,
        882 => Weapon::Minigun,   // Rust Botkiller Minigun Mk.I
        891 => Weapon::Minigun,   // Blood Botkiller Minigun Mk.I
        900 => Weapon::Minigun,   // Carbonado Botkiller Minigun Mk.I
        909 => Weapon::Minigun,   // Diamond Botkiller Minigun Mk.I
        958 => Weapon::Minigun,   // Silver Botkiller Minigun Mk.II
        967 => Weapon::Minigun,   // Gold Botkiller Minigun Mk.II
        15004 => Weapon::Minigun, // King of the Jungle
        15020 => Weapon::Minigun, // Iron Wood
        15026 => Weapon::Minigun, // Antique Annihilator
        15031 => Weapon::Minigun, // War Room
        15040 => Weapon::Minigun, // Citizen Pain
        15055 => Weapon::Minigun, // Brick House
        15086 => Weapon::Minigun, // Macabre Web
        15087 => Weapon::Minigun, // Pumpkin Patch
        15088 => Weapon::Minigun, // Nutcracker
        15098 => Weapon::Minigun, // Brain Candy
        15099 => Weapon::Minigun, // Mister Cuddles
        15123 => Weapon::Minigun, // Coffin Nail
        15124 => Weapon::Minigun, // Dressed to Kill
        15125 => Weapon::Minigun, // Top Shelf
        15147 => Weapon::Minigun, // Butcher Bird

        11 => Weapon::Shotgun,
        42 => Weapon::Sandvich,
        159 => Weapon::DalokohsBar,
        311 => Weapon::BuffaloSteakSandvich,
        425 => Weapon::FamilyBusiness,
        433 => Weapon::Fishcake,
        863 => Weapon::RoboSandvich,
        1002 => Weapon::Sandvich, // Festive
        1190 => Weapon::SecondBanana,

        5 => Weapon::Fists,
        195 => Weapon::Fists, // Renamed/Strange
        43 => Weapon::KillingGlovesOfBoxing,
        239 => Weapon::GlovesOfRunningUrgently,
        310 => Weapon::WarriorsSpirit,
        331 => Weapon::FistsOfSteel,
        426 => Weapon::EvictionNotice,
        587 => Weapon::ApocoFists,
        656 => Weapon::HolidayPunch,
        1084 => Weapon::GlovesOfRunningUrgently, // Festive
        1100 => Weapon::BreadBite,
        1184 => Weapon::GlovesOfRunningUrgently, // MvM

        9 => Weapon::Shotgun,
        141 => Weapon::FrontierJustice,
        527 => Weapon::Widowmaker,
        588 => Weapon::Pomson6000,
        997 => Weapon::RescueRanger,
        1004 => Weapon::FrontierJustice, // Festive

        22 => Weapon::Pistol,
        140 => Weapon::Wrangler,
        528 => Weapon::ShortCircuit,
        1086 => Weapon::Wrangler, // Festive
        30668 => Weapon::GigarCounter,

        7 => Weapon::Wrench,
        197 => Weapon::Wrench, // Renamed/Strange
        142 => Weapon::Gunslinger,
        155 => Weapon::SouthernHospitality,
        169 => Weapon::GoldenWrench,
        329 => Weapon::Jag,
        589 => Weapon::EurekaEffect,
        662 => Weapon::Wrench,   // Festive
        795 => Weapon::Wrench,   // Silver Botkiller Wrench Mk.I
        804 => Weapon::Wrench,   // Gold Botkiller Wrench Mk.I
        884 => Weapon::Wrench,   // Rust Botkiller Wrench Mk.I
        893 => Weapon::Wrench,   // Blood Botkiller Wrench Mk.I
        902 => Weapon::Wrench,   // Carbonado Botkiller Wrench Mk.I
        911 => Weapon::Wrench,   // Diamond Botkiller Wrench Mk.I
        960 => Weapon::Wrench,   // Silver Botkiller Wrench Mk.II
        969 => Weapon::Wrench,   // Gold Botkiller Wrench Mk.II
        15073 => Weapon::Wrench, // Nutcracker
        15074 => Weapon::Wrench, // Autumn
        15075 => Weapon::Wrench, // Boneyard
        15139 => Weapon::Wrench, // Dressed to Kill
        15140 => Weapon::Wrench, // Top Shelf
        15114 => Weapon::Wrench, // Torqued to Hell
        15156 => Weapon::Wrench, // Airwolf

        25 => Weapon::ConstructionPDA,
        737 => Weapon::ConstructionPDA, // Renamed/Strange

        26 => Weapon::DestructionPDA,

        28 => Weapon::Toolbox,

        17 => Weapon::SyringeGun,
        204 => Weapon::SyringeGun, // Renamed/Strange
        36 => Weapon::Blutsauger,
        305 => Weapon::CrusadersCrossbow,
        412 => Weapon::Overdose,
        1079 => Weapon::CrusadersCrossbow, // Festive

        29 => Weapon::MediGun,
        211 => Weapon::MediGun, // Renamed/Strange
        35 => Weapon::Kritzkrieg,
        411 => Weapon::QuickFix,
        663 => Weapon::MediGun, // Festive
        796 => Weapon::MediGun, // Silver Botkiller Medi Gun Mk.I
        805 => Weapon::MediGun, // Gold Botkiller Medi Gun Mk.I
        885 => Weapon::MediGun, // Rust Botkiller Medi Gun Mk.I
        894 => Weapon::MediGun, // Blood Botkiller Medi Gun Mk.I
        903 => Weapon::MediGun, // Carbonado Botkiller Medi Gun Mk.I
        912 => Weapon::MediGun, // Diamond Botkiller Medi Gun Mk.I
        961 => Weapon::MediGun, // Silver Botkiller Medi Gun Mk.II
        970 => Weapon::MediGun, // Gold Botkiller Medi Gun Mk.II
        998 => Weapon::Vaccinator,
        15008 => Weapon::MediGun, // Masked Mender
        15010 => Weapon::MediGun, // Wrapped Reviver
        15025 => Weapon::MediGun, // Reclaimed Reanimator
        15039 => Weapon::MediGun, // Civil Servant
        15050 => Weapon::MediGun, // Spark of Life
        15078 => Weapon::MediGun, // Wildwood
        15097 => Weapon::MediGun, // Flower Power
        15120 => Weapon::MediGun, // Coffin Nail (this one is incorrectly listed in the wiki as 15123)
        15121 => Weapon::MediGun, // Dressed To Kill
        15122 => Weapon::MediGun, // High Roller's
        15145 => Weapon::MediGun, // Blitzkrieg
        15146 => Weapon::MediGun, // Corsair

        8 => Weapon::Bonesaw,
        198 => Weapon::Bonesaw, // Renamed/Strange
        37 => Weapon::Ubersaw,
        173 => Weapon::VitaSaw,
        304 => Weapon::Amputator,
        413 => Weapon::SolemnVow,
        1003 => Weapon::Ubersaw, // Festive
        1143 => Weapon::Bonesaw, // Festive

        14 => Weapon::SniperRifle,
        201 => Weapon::SniperRifle, // Renamed/Strange
        56 => Weapon::Huntsman,
        230 => Weapon::SydneySleeper,
        402 => Weapon::BazaarBargain,
        526 => Weapon::Machina,
        664 => Weapon::SniperRifle, // Festive
        752 => Weapon::HitmansHeatmaker,
        792 => Weapon::SniperRifle, // Silver Botkiller Sniper Rifle Mk.I
        801 => Weapon::SniperRifle, // Gold Botkiller Sniper Rifle Mk.I
        851 => Weapon::AWPerHand,
        881 => Weapon::SniperRifle, // Rust Botkiller Sniper Rifle Mk.I
        890 => Weapon::SniperRifle, // Blood Botkiller Sniper Rifle Mk.I
        899 => Weapon::SniperRifle, // Carbonado Botkiller Sniper Rifle Mk.I
        908 => Weapon::SniperRifle, // Diamond Botkiller Sniper Rifle Mk.I
        957 => Weapon::SniperRifle, // Silver Botkiller Sniper Rifle Mk.II
        966 => Weapon::SniperRifle, // Gold Botkiller Sniper Rifle Mk.II
        1005 => Weapon::Huntsman,   // Festive
        1092 => Weapon::FortifiedCompound,
        1098 => Weapon::Classic,
        15000 => Weapon::SniperRifle,  // Night Owl
        15007 => Weapon::SniperRifle,  // Purple Range
        15019 => Weapon::SniperRifle,  // Lumber From Down Under
        15023 => Weapon::SniperRifle,  // Shot in the Dark
        15033 => Weapon::SniperRifle,  // Bogtrotter
        15059 => Weapon::SniperRifle,  // Thunderbolt
        15070 => Weapon::SniperRifle,  // Pumpkin Patch
        15071 => Weapon::SniperRifle,  // Boneyard
        15072 => Weapon::SniperRifle,  // Wildwood
        15111 => Weapon::SniperRifle,  // Balloonicorn
        15112 => Weapon::SniperRifle,  // Rainbow
        15135 => Weapon::SniperRifle,  // Coffin Nail
        15136 => Weapon::SniperRifle,  // Dressed to Kill
        15154 => Weapon::SniperRifle,  // Airwolf
        30665 => Weapon::ShootingStar, // Shooting Star

        16 => Weapon::SMG,
        203 => Weapon::SMG, // Renamed/Strange
        57 => Weapon::Razorback,
        58 => Weapon::Jarate,
        231 => Weapon::DarwinsDangerShield,
        642 => Weapon::CozyCamper,
        751 => Weapon::CleanersCarbine,
        1083 => Weapon::Jarate, // Festive
        1105 => Weapon::SelfAwareBeautyMark,
        1149 => Weapon::SMG,  // Festive
        15001 => Weapon::SMG, // Woodsy Widowmaker
        15022 => Weapon::SMG, // Plaid Potshotter
        15032 => Weapon::SMG, // Treadplate Tormenter
        15037 => Weapon::SMG, // Team Sprayer
        15058 => Weapon::SMG, // Low Profile
        15076 => Weapon::SMG, // Wildwood
        15110 => Weapon::SMG, // Blue Mew
        15134 => Weapon::SMG, // High Roller's
        15153 => Weapon::SMG, // Blitzkrieg

        3 => Weapon::Kukri,
        193 => Weapon::Kukri, // Renamed/Strange
        171 => Weapon::TribalmansShiv,
        232 => Weapon::Bushwacka,
        401 => Weapon::Shahanshah,

        24 => Weapon::Revolver,
        210 => Weapon::Revolver, // Renamed/Strange
        61 => Weapon::Ambassador,
        161 => Weapon::BigKill,
        224 => Weapon::LEtranger,
        460 => Weapon::Enforcer,
        525 => Weapon::Diamondback,
        1006 => Weapon::Ambassador, // Festive
        1142 => Weapon::Revolver,   // Festive
        15011 => Weapon::Revolver,  // Psychedelic Slugger
        15027 => Weapon::Revolver,  // Old Country
        15042 => Weapon::Revolver,  // Mayor
        15051 => Weapon::Revolver,  // Dead Reckoner
        15062 => Weapon::Revolver,  // Boneyard
        15063 => Weapon::Revolver,  // Wildwood
        15064 => Weapon::Revolver,  // Macabre Web
        15103 => Weapon::Revolver,  // Flower Power
        15127 => Weapon::Revolver,  // Coffin Nail (this one is incorrectly listed in the wiki as 15129)
        15128 => Weapon::Revolver,  // Top Shelf
        15149 => Weapon::Revolver,  // Blitzkrieg

        735 => Weapon::Sapper,
        736 => Weapon::Sapper, // Renamed/Strange
        810 => Weapon::RedTapeRecorder,
        831 => Weapon::RedTapeRecorder, // Genuine
        933 => Weapon::ApSap,           // Genuine
        1080 => Weapon::Sapper,         // Festive
        1102 => Weapon::SnackAttack,

        4 => Weapon::Knife,
        194 => Weapon::Knife, // Renamed/Strange
        225 => Weapon::YourEternalReward,
        356 => Weapon::ConniversKunai,
        461 => Weapon::BigEarner,
        574 => Weapon::WangaPrick,
        638 => Weapon::SharpDresser,
        649 => Weapon::Spycicle,
        665 => Weapon::Knife, // Festive
        727 => Weapon::BlackRose,
        794 => Weapon::Knife,   // Silver Botkiller Knife Mk.I
        803 => Weapon::Knife,   // Gold Botkiller Knife Mk.I
        883 => Weapon::Knife,   // Rust Botkiller Knife Mk.I
        892 => Weapon::Knife,   // Blood Botkiller Knife Mk.I
        901 => Weapon::Knife,   // Carbonado Botkiller Knife Mk.I
        910 => Weapon::Knife,   // Diamond Botkiller Knife Mk.I
        959 => Weapon::Knife,   // Silver Botkiller Knife Mk.II
        968 => Weapon::Knife,   // Gold Botkiller Knife Mk.II
        15080 => Weapon::Knife, // Boneyard (this one is incorrectly listed in the wiki as 15062)
        15094 => Weapon::Knife, // Blue Mew
        15095 => Weapon::Knife, // Brain Candy
        15096 => Weapon::Knife, // Stabbed to Hell
        15118 => Weapon::Knife, // Dressed to Kill
        15119 => Weapon::Knife, // Top Shelf
        15143 => Weapon::Knife, // Blitzkrieg
        15144 => Weapon::Knife, // Airwolf

        27 => Weapon::DisguiseKitPDA,

        30 => Weapon::InvisWatch,
        212 => Weapon::InvisWatch, // Renamed/Strange
        59 => Weapon::DeadRinger,
        60 => Weapon::CloakAndDagger,
        297 => Weapon::EnthusiastsTimepiece,
        947 => Weapon::Quackenbirdt,

        _ => Weapon::Unknown,
    }
}
