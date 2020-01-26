use std::collections::HashMap;


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
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
    Untokenized(u8)
}


pub struct GwTokenInfo {
    token_text : HashMap<String, GwBasicToken>
}

impl GwTokenInfo {
    pub fn create() -> GwTokenInfo {
        let mut dict  = HashMap::new();
        dict.insert(String::from("END"), GwBasicToken::EndTok);
        dict.insert(String::from("+"), GwBasicToken::PlusTok);
        dict.insert(String::from("-"), GwBasicToken::MinusTok);
        dict.insert(String::from("*"), GwBasicToken::TimesTok);
        
        
        GwTokenInfo {
            token_text: dict
        }
    }

    pub fn get_token(&self, tok_text : &String) -> Option<&GwBasicToken> {
        self.token_text.get(tok_text)
    }
}
