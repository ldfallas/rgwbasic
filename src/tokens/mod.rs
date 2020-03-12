use std::collections::HashMap;


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Hash)]
#[derive(Eq)]
pub enum GwBasicToken {
    EndTok,
    ForTok,
    NextTok, 
    DataTok, 
    InputTok,
    DimTok,
    ReadTok,
    LetTok,
    GotoTok,
    RunTok,
    IfTok,
    RestoreTok,
    GosubTok,
    ReturnTok,
    RemTok,
    StopTok,
    PrintTok,
    ClearTok,
    ListTok,
    NewTok,
    OnTok,
    WaitTok,
    DefTok,
    PokeTok,
    ContTok,
    OutTok,
    LprintTok,
    LlistTok,
    WidthTok,
    ElseTok,
    TronTok,
    TroffTok,
    SwapTok,
    EraseTok,
    EditTok,
    ErrorTok,
    ResumeTok,
    DeleteTok,
    AutoTok,
    RenumTok,
    DefstrTok,
    DefintTok,
    DefsngTok,
    DefdblTok,
    LineTok,
    WhileTok,
    WendTok,
    CallTok,
    WriteTok,
    OptionTok,
    RandomizeTok,
    OpenTok,
    CloseTok,
    LoadTok,
    MergeTok,
    SaveTok,
    ColorTok,
    ClsTok,
    MotorTok,
    BsaveTok,
    BloadTok,
    SoundTok,
    BeepTok,
    PsetTok,
    PresetTok,
    ScreenTok,
    KeyTok,
    LocateTok,
    ToTok,
    ThenTok,
    TabTok,
    StepTok,
    UsrTok,
    FnTok,
    SpcTok,
    NotTok,
    ErlTok,
    ErrTok,
    StringDTok,
    UsingTok,
    InstrTok,
    SingleQuoteTok,
    VarptrTok,
    CsrlinTok,
    PointTok,
    OffTok,
    InkeyDTok,
    GtTok,
    EqlTok,
    LtTok,
    PlusTok,
    MinusTok,
    TimesTok,
    DivTok,
    PowOperatorTok,
    AndTok,
    OrTok,
    XorTok,
    EqvTok,
    ImpTok,
    ModTok,
    Div2Tok,
    CviTok,
    CvsTok,
    CvdTok,
    MkiDTok,
    MksDTok,
    MkdDTok,
    ExterrTok,
    FilesTok,
    FieldTok,
    SystemTok,
    NameTok,
    LsetTok,
    RsetTok,
    KillTok,
    PutTok,
    GetTok,
    ResetTok,
    CommonTok,
    ChainTok,
    DateDTok,
    TimeDTok,
    PaintTok,
    ComTok,
    CircleTok,
    DrawTok,
    PlayTok,
    TimerTok,
    ErdevTok,
    IoctlTok,
    ChdirTok,
    MkdirTok,
    RmdirTok,
    ShellTok,
    EnvironTok,
    ViewTok,
    WindowTok,
    PmapTok,
    PaletteTok,
    LcopyTok,
    CallsTok,
    NoiseTok,
    PcopyTok,
    TermTok,
    LockTok,
    UnlockTok,
    LeftDTok,
    RightDTok,
    MidDTok,
    SgnTok,
    IntTok,
    AbsTok,
    SqrTok,
    RndTok,
    SinTok,
    LogTok,
    ExpTok,
    CosTok,
    TanTok,
    AtnTok,
    FreTok,
    InpTok,
    PosTok,
    LenTok,
    StrDTok,
    ValTok,
    AscTok,
    ChrDTok,
    PeekTok,
    SpaceDTok,
    OctDTok,
    HexDTok,
    LposTok,
    CintTok,
    CsngTok,
    CdblTok,
    FixTok,
    PenTok,
    StickTok,
    StrigTok,
    EofTok,
    LocTok,
    LofTok,
    // new tokens
    ColonSeparatorTok,
    CommaSeparatorTok,
    Untokenized(u8)
}


pub struct GwTokenInfo {
    token_text : HashMap<String, GwBasicToken>,
    token_vs_text : HashMap<GwBasicToken, String>        
}

impl GwTokenInfo {
    pub fn create() -> GwTokenInfo {
        let mut dict  = HashMap::new();
        let mut dict2  = HashMap::new();
        GwTokenInfo::add_token("GOTO", GwBasicToken::GotoTok, &mut dict, &mut dict2);
        GwTokenInfo::add_token("END", GwBasicToken::EndTok, &mut dict, &mut dict2);
        GwTokenInfo::add_token("PRINT", GwBasicToken::PrintTok, &mut dict, &mut dict2);
        GwTokenInfo::add_token("INPUT", GwBasicToken::InpTok, &mut dict, &mut dict2);
        GwTokenInfo::add_token("KEY", GwBasicToken::KeyTok, &mut dict, &mut dict2);
        GwTokenInfo::add_token("OFF", GwBasicToken::OffTok, &mut dict, &mut dict2);
        GwTokenInfo::add_token("ON", GwBasicToken::OnTok, &mut dict, &mut dict2);
        GwTokenInfo::add_token("CLS", GwBasicToken::ClsTok, &mut dict, &mut dict2);
        GwTokenInfo::add_token("COLOR", GwBasicToken::ColorTok, &mut dict, &mut dict2);
                
         GwTokenInfo::add_token("*", GwBasicToken::TimesTok, &mut dict, &mut dict2);
         GwTokenInfo::add_token("-", GwBasicToken::MinusTok, &mut dict, &mut dict2);
         GwTokenInfo::add_token("+", GwBasicToken::PlusTok, &mut dict, &mut dict2);

        
        GwTokenInfo {
            token_text: dict,
            token_vs_text: dict2
        }
    }

    fn add_token(tok_text : &str,
                 token : GwBasicToken,
                 txt_vs_token : &mut HashMap<String, GwBasicToken>,
                 token_vs_txt : &mut HashMap<GwBasicToken, String>) {
        let str_key = String::from(tok_text);
        token_vs_txt.insert(token.clone(), str_key);
        // Controversal! couldn't figure out how to reuse the
        // `String` instance created above without adding a lifetime annotation
        // to this struct which makes using this struct very difficult
        txt_vs_token.insert(String::from(tok_text), token);
    }

    pub fn get_token(&self, tok_text : &String) -> Option<&GwBasicToken> {
        self.token_text.get(tok_text)
    }
}
