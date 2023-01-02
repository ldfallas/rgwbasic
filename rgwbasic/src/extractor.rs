use std::io::Read;
use std::io::Cursor;

use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap; 



fn add_token( byte1 : u8, byte2 : u8, tok : GwBasicToken, dict : &mut HashMap<u8, HashMap<u8, GwBasicToken>>) {
    let inner :  &mut HashMap<u8, GwBasicToken>;

    if !dict.contains_key(&byte1) {
        dict.insert(byte1, HashMap::new());
    }
    
    if let Some(existing_dict) = dict.get_mut(&byte1) {
        inner = existing_dict;
    } else {
        panic!("error getting inner dictionary");
    }
    inner.insert(byte2, tok);
}

fn init_multi_byte_tokens() -> HashMap<u8, HashMap<u8, GwBasicToken>> {
    let mut dict = HashMap::new();
add_token(0xFD, 0x81, GwBasicToken::CviTok,&mut dict);
add_token(0xFD, 0x82, GwBasicToken::CvsTok,&mut dict);
add_token(0xFD, 0x83, GwBasicToken::CvdTok,&mut dict);
add_token(0xFD, 0x84, GwBasicToken::MkiDTok,&mut dict);
add_token(0xFD, 0x85, GwBasicToken::MksDTok,&mut dict);
add_token(0xFD, 0x86, GwBasicToken::MkdDTok,&mut dict);
add_token(0xFD, 0x8B, GwBasicToken::ExterrTok,&mut dict);
add_token(0xFE, 0x81, GwBasicToken::FilesTok,&mut dict);
add_token(0xFE, 0x82, GwBasicToken::FieldTok,&mut dict);
add_token(0xFE, 0x83, GwBasicToken::SystemTok,&mut dict);
add_token(0xFE, 0x84, GwBasicToken::NameTok,&mut dict);
add_token(0xFE, 0x85, GwBasicToken::LsetTok,&mut dict);
add_token(0xFE, 0x86, GwBasicToken::RsetTok,&mut dict);
add_token(0xFE, 0x87, GwBasicToken::KillTok,&mut dict);
add_token(0xFE, 0x88, GwBasicToken::PutTok,&mut dict);
add_token(0xFE, 0x89, GwBasicToken::GetTok,&mut dict);
add_token(0xFE, 0x8A, GwBasicToken::ResetTok,&mut dict);
add_token(0xFE, 0x8B, GwBasicToken::CommonTok,&mut dict);
add_token(0xFE, 0x8C, GwBasicToken::ChainTok,&mut dict);
add_token(0xFE, 0x8D, GwBasicToken::DateDTok,&mut dict);
add_token(0xFE, 0x8E, GwBasicToken::TimeDTok,&mut dict);
add_token(0xFE, 0x8F, GwBasicToken::PaintTok,&mut dict);
add_token(0xFE, 0x90, GwBasicToken::ComTok,&mut dict);
add_token(0xFE, 0x91, GwBasicToken::CircleTok,&mut dict);
add_token(0xFE, 0x92, GwBasicToken::DrawTok,&mut dict);
add_token(0xFE, 0x93, GwBasicToken::PlayTok,&mut dict);
add_token(0xFE, 0x94, GwBasicToken::TimerTok,&mut dict);
add_token(0xFE, 0x95, GwBasicToken::ErdevTok,&mut dict);
add_token(0xFE, 0x96, GwBasicToken::IoctlTok,&mut dict);
add_token(0xFE, 0x97, GwBasicToken::ChdirTok,&mut dict);
add_token(0xFE, 0x98, GwBasicToken::MkdirTok,&mut dict);
add_token(0xFE, 0x99, GwBasicToken::RmdirTok,&mut dict);
add_token(0xFE, 0x9A, GwBasicToken::ShellTok,&mut dict);
add_token(0xFE, 0x9B, GwBasicToken::EnvironTok,&mut dict);
add_token(0xFE, 0x9C, GwBasicToken::ViewTok,&mut dict);
add_token(0xFE, 0x9D, GwBasicToken::WindowTok,&mut dict);
add_token(0xFE, 0x9E, GwBasicToken::PmapTok,&mut dict);
add_token(0xFE, 0x9F, GwBasicToken::PaletteTok,&mut dict);
add_token(0xFE, 0xA0, GwBasicToken::LcopyTok,&mut dict);
add_token(0xFE, 0xA1, GwBasicToken::CallsTok,&mut dict);
add_token(0xFE, 0xA4, GwBasicToken::NoiseTok,&mut dict);
add_token(0xFE, 0xA5, GwBasicToken::PcopyTok,&mut dict);
add_token(0xFE, 0xA6, GwBasicToken::TermTok,&mut dict);
add_token(0xFE, 0xA7, GwBasicToken::LockTok,&mut dict);
add_token(0xFE, 0xA8, GwBasicToken::UnlockTok,&mut dict);
add_token(0xFF, 0x81, GwBasicToken::LeftDTok,&mut dict);
add_token(0xFF, 0x82, GwBasicToken::RightDTok,&mut dict);
add_token(0xFF, 0x83, GwBasicToken::MidDTok,&mut dict);
add_token(0xFF, 0x84, GwBasicToken::SgnTok,&mut dict);
add_token(0xFF, 0x85, GwBasicToken::IntTok,&mut dict);
add_token(0xFF, 0x86, GwBasicToken::AbsTok,&mut dict);
add_token(0xFF, 0x87, GwBasicToken::SqrTok,&mut dict);
add_token(0xFF, 0x88, GwBasicToken::RndTok,&mut dict);
add_token(0xFF, 0x89, GwBasicToken::SinTok,&mut dict);
add_token(0xFF, 0x8A, GwBasicToken::LogTok,&mut dict);
add_token(0xFF, 0x8B, GwBasicToken::ExpTok,&mut dict);
add_token(0xFF, 0x8C, GwBasicToken::CosTok,&mut dict);
add_token(0xFF, 0x8D, GwBasicToken::TanTok,&mut dict);
add_token(0xFF, 0x8E, GwBasicToken::AtnTok,&mut dict);
add_token(0xFF, 0x8F, GwBasicToken::FreTok,&mut dict);
add_token(0xFF, 0x90, GwBasicToken::InpTok,&mut dict);
add_token(0xFF, 0x91, GwBasicToken::PosTok,&mut dict);
add_token(0xFF, 0x92, GwBasicToken::LenTok,&mut dict);
add_token(0xFF, 0x93, GwBasicToken::StrDTok,&mut dict);
add_token(0xFF, 0x94, GwBasicToken::ValTok,&mut dict);
add_token(0xFF, 0x95, GwBasicToken::AscTok,&mut dict);
add_token(0xFF, 0x96, GwBasicToken::ChrDTok,&mut dict);
add_token(0xFF, 0x97, GwBasicToken::PeekTok,&mut dict);
add_token(0xFF, 0x98, GwBasicToken::SpaceDTok,&mut dict);
add_token(0xFF, 0x99, GwBasicToken::OctDTok,&mut dict);
add_token(0xFF, 0x9A, GwBasicToken::HexDTok,&mut dict);
add_token(0xFF, 0x9B, GwBasicToken::LposTok,&mut dict);
add_token(0xFF, 0x9C, GwBasicToken::CintTok,&mut dict);
add_token(0xFF, 0x9D, GwBasicToken::CsngTok,&mut dict);
add_token(0xFF, 0x9E, GwBasicToken::CdblTok,&mut dict);
add_token(0xFF, 0x9F, GwBasicToken::FixTok,&mut dict);
add_token(0xFF, 0xA0, GwBasicToken::PenTok,&mut dict);
add_token(0xFF, 0xA1, GwBasicToken::StickTok,&mut dict);
add_token(0xFF, 0xA2, GwBasicToken::StrigTok,&mut dict);
add_token(0xFF, 0xA3, GwBasicToken::EofTok,&mut dict);
add_token(0xFF, 0xA4, GwBasicToken::LocTok,&mut dict);
add_token(0xFF, 0xA5, GwBasicToken::LofTok,&mut dict);

    dict
}

fn init_single_byte_tokens() -> HashMap<u8, GwBasicToken> {
    let mut single_byte_tokens = HashMap::new();
    single_byte_tokens.insert(0x81, GwBasicToken::EndTok);
    single_byte_tokens.insert(0x82, GwBasicToken::ForTok);
    single_byte_tokens.insert(0x83, GwBasicToken::NextTok);
    single_byte_tokens.insert(0x84, GwBasicToken::DataTok);
    single_byte_tokens.insert(0x85, GwBasicToken::InputTok);
    single_byte_tokens.insert(0x86, GwBasicToken::DimTok);
    single_byte_tokens.insert(0x87, GwBasicToken::ReadTok);
    single_byte_tokens.insert(0x88, GwBasicToken::LetTok);
    single_byte_tokens.insert(0x89, GwBasicToken::GotoTok);
    single_byte_tokens.insert(0x8A, GwBasicToken::RunTok);
    single_byte_tokens.insert(0x8B, GwBasicToken::IfTok);
    single_byte_tokens.insert(0x8C, GwBasicToken::RestoreTok);
    single_byte_tokens.insert(0x8D, GwBasicToken::GosubTok);
    single_byte_tokens.insert(0x8E, GwBasicToken::ReturnTok);
    single_byte_tokens.insert(0x8F, GwBasicToken::RemTok);
    single_byte_tokens.insert(0x90, GwBasicToken::StopTok);
    single_byte_tokens.insert(0x91, GwBasicToken::PrintTok);
    single_byte_tokens.insert(0x92, GwBasicToken::ClearTok);
    single_byte_tokens.insert(0x93, GwBasicToken::ListTok);
    single_byte_tokens.insert(0x94, GwBasicToken::NewTok);
    single_byte_tokens.insert(0x95, GwBasicToken::OnTok);
    single_byte_tokens.insert(0x96, GwBasicToken::WaitTok);
    single_byte_tokens.insert(0x97, GwBasicToken::DefTok);
    single_byte_tokens.insert(0x98, GwBasicToken::PokeTok);
    single_byte_tokens.insert(0x99, GwBasicToken::ContTok);
    single_byte_tokens.insert(0x9C, GwBasicToken::OutTok);
    single_byte_tokens.insert(0x9D, GwBasicToken::LprintTok);
    single_byte_tokens.insert(0x9E, GwBasicToken::LlistTok);
    single_byte_tokens.insert(0xA0, GwBasicToken::WidthTok);
    single_byte_tokens.insert(0xA1, GwBasicToken::ElseTok);
    single_byte_tokens.insert(0xA2, GwBasicToken::TronTok);
    single_byte_tokens.insert(0xA3, GwBasicToken::TroffTok);
    single_byte_tokens.insert(0xA4, GwBasicToken::SwapTok);
    single_byte_tokens.insert(0xA5, GwBasicToken::EraseTok);
    single_byte_tokens.insert(0xA6, GwBasicToken::EditTok);
    single_byte_tokens.insert(0xA7, GwBasicToken::ErrorTok);
    single_byte_tokens.insert(0xA8, GwBasicToken::ResumeTok);
    single_byte_tokens.insert(0xA9, GwBasicToken::DeleteTok);
    single_byte_tokens.insert(0xAA, GwBasicToken::AutoTok);
    single_byte_tokens.insert(0xAB, GwBasicToken::RenumTok);
    single_byte_tokens.insert(0xAC, GwBasicToken::DefstrTok);
    single_byte_tokens.insert(0xAD, GwBasicToken::DefintTok);
    single_byte_tokens.insert(0xAE, GwBasicToken::DefsngTok);
    single_byte_tokens.insert(0xAF, GwBasicToken::DefdblTok);
    single_byte_tokens.insert(0xB0, GwBasicToken::LineTok);
    single_byte_tokens.insert(0xB1, GwBasicToken::WhileTok);
    single_byte_tokens.insert(0xB2, GwBasicToken::WendTok);
    single_byte_tokens.insert(0xB3, GwBasicToken::CallTok);
    single_byte_tokens.insert(0xB7, GwBasicToken::WriteTok);
    single_byte_tokens.insert(0xB8, GwBasicToken::OptionTok);
    single_byte_tokens.insert(0xB9, GwBasicToken::RandomizeTok);
    single_byte_tokens.insert(0xBA, GwBasicToken::OpenTok);
    single_byte_tokens.insert(0xBB, GwBasicToken::CloseTok);
    single_byte_tokens.insert(0xBC, GwBasicToken::LoadTok);
    single_byte_tokens.insert(0xBD, GwBasicToken::MergeTok);
    single_byte_tokens.insert(0xBE, GwBasicToken::SaveTok);
    single_byte_tokens.insert(0xBF, GwBasicToken::ColorTok);
    single_byte_tokens.insert(0xC0, GwBasicToken::ClsTok);
    single_byte_tokens.insert(0xC1, GwBasicToken::MotorTok);
    single_byte_tokens.insert(0xC2, GwBasicToken::BsaveTok);
    single_byte_tokens.insert(0xC3, GwBasicToken::BloadTok);
    single_byte_tokens.insert(0xC4, GwBasicToken::SoundTok);
    single_byte_tokens.insert(0xC5, GwBasicToken::BeepTok);
    single_byte_tokens.insert(0xC6, GwBasicToken::PsetTok);
    single_byte_tokens.insert(0xC7, GwBasicToken::PresetTok);
    single_byte_tokens.insert(0xC8, GwBasicToken::ScreenTok);
    single_byte_tokens.insert(0xC9, GwBasicToken::KeyTok);
    single_byte_tokens.insert(0xCA, GwBasicToken::LocateTok);
    single_byte_tokens.insert(0xCC, GwBasicToken::ToTok);
    single_byte_tokens.insert(0xCD, GwBasicToken::ThenTok);
    single_byte_tokens.insert(0xCE, GwBasicToken::TabTok);
    single_byte_tokens.insert(0xCF, GwBasicToken::StepTok);
    single_byte_tokens.insert(0xD0, GwBasicToken::UsrTok);
    single_byte_tokens.insert(0xD1, GwBasicToken::FnTok);
    single_byte_tokens.insert(0xD2, GwBasicToken::SpcTok);
    single_byte_tokens.insert(0xD3, GwBasicToken::NotTok);
    single_byte_tokens.insert(0xD4, GwBasicToken::ErlTok);
    single_byte_tokens.insert(0xD5, GwBasicToken::ErrTok);
    single_byte_tokens.insert(0xD6, GwBasicToken::StringDTok);
    single_byte_tokens.insert(0xD7, GwBasicToken::UsingTok);
    single_byte_tokens.insert(0xD8, GwBasicToken::InstrTok);
    single_byte_tokens.insert(0xD9, GwBasicToken::SingleQuoteTok);
    single_byte_tokens.insert(0xDA, GwBasicToken::VarptrTok);
    single_byte_tokens.insert(0xDB, GwBasicToken::CsrlinTok);
    single_byte_tokens.insert(0xDC, GwBasicToken::PointTok);
    single_byte_tokens.insert(0xDD, GwBasicToken::OffTok);
    single_byte_tokens.insert(0xDE, GwBasicToken::InkeyDTok);
    single_byte_tokens.insert(0xE6, GwBasicToken::GtTok);
    single_byte_tokens.insert(0xE7, GwBasicToken::EqlTok);
    single_byte_tokens.insert(0xE8, GwBasicToken::LtTok);
    single_byte_tokens.insert(0xE9, GwBasicToken::PlusTok);
    single_byte_tokens.insert(0xEA, GwBasicToken::MinusTok);
    single_byte_tokens.insert(0xEB, GwBasicToken::TimesTok);
    single_byte_tokens.insert(0xEC, GwBasicToken::DivTok);
    single_byte_tokens.insert(0xED, GwBasicToken::PowOperatorTok);
    single_byte_tokens.insert(0xEE, GwBasicToken::AndTok);
    single_byte_tokens.insert(0xEF, GwBasicToken::OrTok);
    single_byte_tokens.insert(0xF0, GwBasicToken::XorTok);
    single_byte_tokens.insert(0xF1, GwBasicToken::EqvTok);
    single_byte_tokens.insert(0xF2, GwBasicToken::ImpTok);
    single_byte_tokens.insert(0xF3, GwBasicToken::ModTok);
    single_byte_tokens.insert(0xF4, GwBasicToken::Div2Tok);
    single_byte_tokens
}

fn read_u16(reader : &mut dyn Read) -> u16 {
    let mut two_bytes = [0; 2];
    reader.read(&mut two_bytes).expect("error");
    let result : u16 = (u16::from(two_bytes[1]) << 8)
                       | (u16::from(two_bytes[0]));
    result
}

fn recognize_tokens(reader : &mut dyn Read, dict : &HashMap<u8, GwBasicToken>) {
    let mut item = [0; 1];
    let multi_token_dict = init_multi_byte_tokens();

    let mut secondary_header = [0; 4];
    reader.read(&mut secondary_header);

    
    loop {
        let read_bytes = reader.read(&mut item).expect("Error reading");
        if read_bytes == 0 {
            break;
        }
        if let Some(tok) = dict.get(&item[0]) {
            println!("Recognized: {:?}, {}", tok, item[0]);            
        } else if item[0] >= 0x20 && item[0] <= 0x7E {
            println!("Unrecognized {}", char::from(item[0]));
        } else if item[0] == 0xF {
            reader.read(&mut item).expect("Error reading");
            println!("Single byte constant {}", item[0] );
        } else if item[0] == 0x1C {
            let two_byte_constant = read_u16(reader);            
            println!("Two byte constant {}", two_byte_constant );
        } else if item[0] >= 0xf5 && item[0] <= 0xFC {
           println!("Undefined {}", item[0]);
        } else if item[0] == 0x1D {
            let mut floating_number = [0; 4];
            reader.read(&mut floating_number);
            println!("Floating point literal ...");
        } else if item[0] == 0xE {
            let gosub_line = read_u16( reader );
            println!("Line reference {}", gosub_line);
        } else if item[0] >= 0x11 && item[0] <= 0x1B {
            println!("Number constant {}", item[0] - 0x11);
        } else if let Some(inner_dict) = multi_token_dict.get(&item[0]) {
            reader.read(&mut item).expect("Error reading");
            if let Some(tok2) = inner_dict.get(&item[0]) {
                println!("Multi recognized {:?}", tok2);
            } else {
                panic!("Unknown multibyte");
            }
        } else if item[0] == 0xC {
              let mut hex_lit = [0; 2];
              reader.read(&mut hex_lit).expect("error");
              println!("Hex literal");       
        } else if item[0] == 0x0 {
            let mut next_line_number = [0; 4];
            reader.read(&mut next_line_number).expect("error");
            let new_offset : u16 = (u16::from(next_line_number[1]) << 8) | (u16::from(next_line_number[0]));
            
            let current_line : u16 = (u16::from(next_line_number[3]) << 8) | (u16::from(next_line_number[2]));
            println!("Line number: {}, {}", current_line, new_offset);
            
        } else {
            println!("About to fail with {:x}", item[0]);
            panic!("Not implemented");
        }        
    }
}

fn main() {
    println!("Experimental extractor");
    let mut file = File::open("IDSLOT.BAS")
    
        .expect("Cannot open file");
    let mut header = [0; 1];
    file.read(&mut header)
        .expect("Cannot read from input file");
    let key_1 : [u8; 13] = [ 0x9A, 0xF7, 0x19, 0x83, 0x24, 0x63,
                             0x43, 0x83, 0x75, 0xCD, 0x8D, 0x84,
                             0xA9 ];
    let key_2 : [u8; 11] = [ 0x7C, 0x88, 0x59, 0x74, 0xE0, 0x97,
                            0x26, 0x77, 0xC4, 0x1D, 0x1E ];
        let toks = init_single_byte_tokens();
    if header[0] == 0xFF {
        println!("File not protected");

        recognize_tokens(&mut file, &toks);
        //match toks.get(&secondary_header[4]) {
        //    Some(tok) => println!("Recognized: {:?}, {}", tok, secondary_header[5]),
        //    None => println!("Token not recognized {}", secondary_header[4])
        //}
        
    } else {
        println!("File protected. Input byte: {}", header[0]);
        let mut contents = vec![];
        let full_file = file.read_to_end(&mut contents)
            .expect("error reading");
        let mut idx1 : u8 = 0x0D;
        let mut idx2 : u8 = 0x0B;
        let mut i = 0;
        while i < contents.len() {
            let sub_result = contents[i].wrapping_sub(idx2);
            let first_xor = sub_result ^ key_1[usize::from(idx1 - 1)];
            let second_xor = first_xor ^ key_2[usize::from(idx2 - 1)];
            contents[i] = second_xor.wrapping_add(idx1);
            idx1 = idx1 - 1;
            idx2 = idx2 - 1;
            if idx2 == 0 {
                idx2 = 0x0B;
            }

            if idx1 == 0 {
                idx1 = 0x0D;
            }
            i = i + 1;
        }
        let off_2 : u16 = (u16::from(contents[1]) << 8) | (u16::from(contents[0]));
        let current_line : u16 = (u16::from(contents[3]) << 8) | (u16::from(contents[2]));
        println!("{} <> {}", off_2, current_line);
        
        recognize_tokens(&mut Cursor::new(contents), &toks);
                                                               
                                                           
                                                           
    }
}
