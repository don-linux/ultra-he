use hekit::actuation::{self, Record};
use hekit::board::Board;
use hekit::dks;
use hekit::endpoint::{Endpoint, MockIo};
use hekit::error::Error;
use hekit::frame::{
    self, DONGLE_FRAME, SET_ACTUATION, SET_COLOR, SET_DKS, SET_FN, SET_GLOBALS, SET_KEYS,
    SET_LIGHT, WIRED_FRAME,
};
use hekit::globals::Globals;
use hekit::keys::{self, KeyRecord};
use hekit::layout;
use hekit::lighting::{self, Lighting};
use hekit::link::{self, Kind};
use hekit::macro_store::{self, Event};
use hekit::profile;
use hekit::switches::SwitchChoice;

fn wired_board() -> Board {
    let mut board = Board::default();
    board.origin = hekit::board::Origin::Device;
    board
}

#[test]
fn request_starts_with_aa() {
    let frame = frame::build_request(WIRED_FRAME, SET_LIGHT, 0, &[0u8; 16], true).unwrap();
    assert_eq!(frame[0], 0xAA);
}

#[test]
fn reply_without_55_is_discarded() {
    let buf = vec![0xAA, SET_LIGHT, 0, 0, 0, 0, 1, 0];
    assert!(frame::parse_reply(&buf, SET_LIGHT, 0).is_err());
}

#[test]
fn only_the_last_chunk_is_flagged() {
    let frames = frame::paginate(WIRED_FRAME, SET_ACTUATION, &[0u8; 1008]).unwrap();
    assert!(frames[..frames.len() - 1].iter().all(|frame| frame[6] == 0));
    assert_eq!(frames.last().unwrap()[6], 1);
}

#[test]
fn offset_0x38_is_little_endian() {
    let frames = frame::paginate(WIRED_FRAME, SET_GLOBALS, &[0u8; 64]).unwrap();
    assert_eq!(&frames[1][3..5], &[0x38, 0x00]);
}

#[test]
fn wired_full_chunk_is_56_and_tail_can_be_shorter() {
    let frames = frame::paginate(WIRED_FRAME, SET_GLOBALS, &[0u8; 64]).unwrap();
    assert_eq!(frames[0][2], 56);
    assert_eq!(frames[1][2], 8);
}

#[test]
fn dongle_full_chunk_is_24() {
    let frames = frame::paginate(DONGLE_FRAME, SET_ACTUATION, &[0u8; 24]).unwrap();
    assert_eq!(frames[0][2], 24);
}

#[test]
fn reply_with_another_command_does_not_match() {
    let mut buf = vec![0u8; 64];
    buf[0] = 0x55;
    buf[1] = 0x10;
    assert!(frame::parse_reply(&buf, 0x17, 0).is_err());
}

#[test]
fn reply_with_another_offset_does_not_match() {
    let mut buf = vec![0u8; 64];
    buf[0] = 0x55;
    buf[1] = 0x17;
    buf[3] = 0x38;
    assert!(frame::parse_reply(&buf, 0x17, 0).is_err());
}

#[test]
fn timeout_is_not_a_payload() {
    let mut mock = MockIo::wired();
    mock.timeouts = 1;
    mock.tables.push((0x17, vec![0u8; 8]));
    let mut endpoint = Endpoint::new(mock);
    let data = endpoint.read_table(0x17, 8).unwrap();
    assert_eq!(data.len(), 8);
    assert!(endpoint.io.frames.len() >= 2);
}

#[test]
fn fatal_read_stops_the_transaction() {
    let mut mock = MockIo::wired();
    mock.fatal = true;
    let mut endpoint = Endpoint::new(mock);
    assert!(endpoint.read_table(0x17, 8).is_err());
}

#[test]
fn three_timeouts_do_not_issue_the_set() {
    let mut mock = MockIo::wired();
    mock.timeouts = 9;
    let mut endpoint = Endpoint::new(mock);
    let error = endpoint.read_modify_write(0x17, SET_ACTUATION, 8, Ok);
    assert!(error.is_err());
    assert!(endpoint
        .io
        .frames
        .iter()
        .all(|frame| frame[1] != SET_ACTUATION));
}

#[test]
fn dongle_refuses_a_64_byte_frame_before_write() {
    let mut endpoint = Endpoint::new(MockIo {
        link: hekit::link::Link::dongle(),
        ..MockIo::wired()
    });
    let frame = vec![0xAA; 64];
    assert!(endpoint.write_frame(&frame).is_err());
    assert!(endpoint.io.frames.is_empty());
}

#[test]
fn wired_refuses_a_32_byte_frame_before_write() {
    let mut endpoint = Endpoint::new(MockIo::wired());
    let frame = vec![0xAA; 32];
    assert!(endpoint.write_frame(&frame).is_err());
    assert!(endpoint.io.frames.is_empty());
}

#[test]
fn wrong_usage_page_is_not_a_link() {
    assert!(link::classify(0x0C45, 0x80A2, 0xFF00, 0x61, "KEYBOARD MINI 60 HE PRO").is_err());
}

#[test]
fn both_links_prefer_the_cable() {
    let chosen = link::prefer(&[hekit::link::Link::dongle(), hekit::link::Link::wired()]).unwrap();
    assert_eq!(chosen.kind, Kind::Wired);
}

#[test]
fn alt_pid_is_refused() {
    assert!(matches!(
        link::classify(0x0C45, 0x80B2, 0xFF68, 0x61, "KEYBOARD MINI 60 HE PRO"),
        Err(Error::RefusedPid)
    ));
}

#[test]
fn trip_below_0_10_is_rejected() {
    let mut record = Record::default();
    assert!(actuation::set_trip(&mut record, SwitchChoice::Axis(2), 9).is_err());
}

#[test]
fn trip_0_10_is_accepted() {
    let mut record = Record::default();
    actuation::set_trip(&mut record, SwitchChoice::Axis(2), 10).unwrap();
    assert_eq!(record.trip, 10);
}

#[test]
fn jade_pro_accepts_3_30() {
    let mut record = Record::default();
    actuation::set_trip(&mut record, SwitchChoice::Axis(2), 330).unwrap();
    assert_eq!(record.axis_type, 2);
}

#[test]
fn jade_pro_rejects_3_31() {
    let mut record = Record::default();
    assert!(actuation::set_trip(&mut record, SwitchChoice::Axis(2), 331).is_err());
    assert_ne!(record.encode()[2..4], 331u16.to_le_bytes());
}

#[test]
fn jade_ultra_accepts_3_20() {
    let mut record = Record::default();
    actuation::set_trip(&mut record, SwitchChoice::JadeUltra, 320).unwrap();
    assert_eq!(record.axis_type, 2);
}

#[test]
fn jade_ultra_rejects_3_21() {
    let mut record = Record::default();
    assert!(actuation::set_trip(&mut record, SwitchChoice::JadeUltra, 321).is_err());
}

#[test]
fn axis_1_caps_at_3_30() {
    assert!(actuation::set_trip(&mut Record::default(), SwitchChoice::Axis(1), 331).is_err());
}

#[test]
fn axis_3_caps_at_3_30() {
    assert!(actuation::set_trip(&mut Record::default(), SwitchChoice::Axis(3), 331).is_err());
}

#[test]
fn axis_4_caps_at_3_40() {
    actuation::set_trip(&mut Record::default(), SwitchChoice::Axis(4), 340).unwrap();
    assert!(actuation::set_trip(&mut Record::default(), SwitchChoice::Axis(4), 341).is_err());
}

#[test]
fn axis_5_caps_at_3_40() {
    assert!(actuation::set_trip(&mut Record::default(), SwitchChoice::Axis(5), 341).is_err());
}

#[test]
fn axis_6_caps_at_3_40() {
    assert!(actuation::set_trip(&mut Record::default(), SwitchChoice::Axis(6), 341).is_err());
}

#[test]
fn axis_7_caps_at_3_40() {
    assert!(actuation::set_trip(&mut Record::default(), SwitchChoice::Axis(7), 341).is_err());
}

#[test]
fn axis_8_caps_at_3_40() {
    assert!(actuation::set_trip(&mut Record::default(), SwitchChoice::Axis(8), 341).is_err());
}

#[test]
fn unknown_axis_9_is_rejected() {
    assert!(SwitchChoice::Axis(9).max_cmm().is_err());
    assert!(SwitchChoice::parse_profile("axis:9").is_err());
}

#[test]
fn negative_millimetres_are_rejected() {
    assert!(actuation::cmm_from_mm_text("-0.10").is_err());
}

#[test]
fn three_millimetres_encode_as_2c_01() {
    let mut record = Record::default();
    actuation::set_trip(&mut record, SwitchChoice::Axis(6), 300).unwrap();
    assert_eq!(&record.encode()[2..4], &[0x2C, 0x01]);
}

#[test]
fn rt_press_zero_is_rejected() {
    assert!(actuation::set_rt_press(&mut Record::default(), SwitchChoice::Axis(2), 0).is_err());
}

#[test]
fn rt_press_0_01_is_accepted() {
    let mut record = Record::default();
    actuation::set_rt_press(&mut record, SwitchChoice::Axis(2), 1).unwrap();
    assert_eq!(record.rt_press, 1);
    assert_ne!(record.flags & 1, 0);
}

#[test]
fn changing_release_keeps_press() {
    let mut board = wired_board();
    board.set_rt_press(0, 40).unwrap();
    board.set_rt_release(0, 25).unwrap();
    assert_eq!(board.actuation[0].rt_press, 40);
    assert_eq!(board.actuation[0].rt_release, 25);
}

#[test]
fn writing_rt_sets_bit0() {
    let mut record = Record::default();
    actuation::set_rt_press(&mut record, SwitchChoice::Axis(6), 20).unwrap();
    assert_eq!(record.flags & actuation::FLAG_RT, actuation::FLAG_RT);
}

#[test]
fn disabling_rt_keeps_distances() {
    let mut record = Record::default();
    actuation::set_rt_press(&mut record, SwitchChoice::Axis(6), 20).unwrap();
    actuation::set_rt_release(&mut record, SwitchChoice::Axis(6), 15).unwrap();
    actuation::set_rt_enabled(&mut record, false);
    assert_eq!(record.rt_press, 20);
    assert_eq!(record.rt_release, 15);
    assert_eq!(record.flags & actuation::FLAG_RT, 0);
}

#[test]
fn editing_trip_keeps_rampage_bit() {
    let mut board = wired_board();
    board.actuation[0].flags |= actuation::FLAG_RAMPAGE;
    board.set_trip(0, 150).unwrap();
    assert_ne!(board.actuation[0].flags & actuation::FLAG_RAMPAGE, 0);
}

#[test]
fn editing_trip_keeps_axis_zero() {
    let mut board = wired_board();
    board.choice[0] = SwitchChoice::Unlisted(0);
    board.actuation[0].axis_type = 0;
    board.set_trip(0, 150).unwrap();
    assert_eq!(board.actuation[0].axis_type, 0);
}

#[test]
fn actuation_table_is_1008_bytes() {
    let bytes = actuation::encode_table(&[Record::default(); actuation::RECORDS]);
    assert_eq!(bytes.len(), 1008);
}

#[test]
fn fn_write_does_not_touch_the_base_map() {
    let mut board = wired_board();
    board.base[0] = keys::keyboard(0, 0x29);
    let before = board.base;
    board.fn_layer[28] = keys::keyboard(0, 0x45);
    let frames =
        frame::paginate(WIRED_FRAME, SET_FN, &keys::fn_write_bytes(&board.fn_layer)).unwrap();
    assert!(frames.iter().all(|frame| frame[1] == SET_FN));
    assert_eq!(board.base, before);
}

#[test]
fn base_write_does_not_touch_the_fn_layer() {
    let mut board = wired_board();
    board.fn_layer[28] = keys::keyboard(0, 0x45);
    let before = board.fn_layer;
    board.base[0] = keys::keyboard(0, 0x04);
    let bytes = keys::base_write_bytes(&board.base).unwrap();
    let frames = frame::paginate(WIRED_FRAME, SET_KEYS, &bytes).unwrap();
    assert!(frames.iter().all(|frame| frame[1] == SET_KEYS));
    assert_eq!(board.fn_layer, before);
}

#[test]
fn base_write_is_504_bytes() {
    let bytes = keys::base_write_bytes(&[KeyRecord::default(); keys::SLOTS]).unwrap();
    assert_eq!(bytes.len(), 504);
}

#[test]
fn fn_write_is_512_bytes() {
    let bytes = keys::fn_write_bytes(&[KeyRecord::default(); keys::SLOTS]);
    assert_eq!(bytes.len(), 512);
}

#[test]
fn page_type_2_keeps_the_hid_usage() {
    let record = keys::keyboard(0, 0x45);
    assert_eq!(record.encode(), [2, 0, 0x45, 0]);
}

#[test]
fn page_type_2_keeps_modifier_0x40() {
    let record = keys::keyboard(0x40, 0);
    assert_eq!(record.encode()[1], 0x40);
}

#[test]
fn page_type_3_keeps_a_little_endian_usage() {
    let record = keys::consumer(0x0223);
    assert_eq!(record.encode(), [3, 0x23, 0x02, 0]);
    assert_eq!(keys::consumer_usage(record), Some(0x0223));
}

#[test]
fn page_type_6_keeps_slot_mode_and_count() {
    assert_eq!(keys::macro_bind(1, 0, 7).encode(), [6, 1, 0, 7]);
}

#[test]
fn page_type_13_keeps_the_function_id() {
    let record = keys::device_function(0x12);
    assert_eq!(keys::function_id(record), Some(0x12));
}

#[test]
fn index_83_is_the_spacebar() {
    let space = layout::find(83).unwrap();
    assert_eq!(space.hid, 0x2C);
    assert_ne!(space.label, "60");
}

#[test]
fn socd_needs_both_keys() {
    let mut records = [KeyRecord::default(); keys::SLOTS];
    assert!(keys::apply_pair(&mut records, 49, 49, keys::socd(3, 0x04, 0x07)).is_err());
    keys::apply_pair(&mut records, 49, 51, keys::socd(3, 0x04, 0x07)).unwrap();
    assert_eq!(records[49], records[51]);
}

#[test]
fn clearing_socd_zeros_both_records() {
    let mut records = [KeyRecord::default(); keys::SLOTS];
    keys::apply_pair(&mut records, 49, 51, keys::socd(3, 0x04, 0x07)).unwrap();
    keys::clear_pair(&mut records, 49, 51).unwrap();
    assert!(records[49].is_empty() && records[51].is_empty());
}

#[test]
fn rapid_snap_needs_both_keys() {
    let mut records = [KeyRecord::default(); keys::SLOTS];
    assert!(keys::apply_pair(&mut records, 33, 33, keys::rapid_snap(0x14, 0x08)).is_err());
}

#[test]
fn dks_1_6_mm_is_16_tenths() {
    assert_eq!(dks::tenths_from_cmm(160).unwrap(), 16);
}

#[test]
fn dks_slot_63_is_rejected() {
    assert!(dks::check_slot(63).is_err());
    assert!(keys::dks_bind(63).is_err());
}

#[test]
fn macro_delay_65535_is_accepted() {
    assert_eq!(macro_store::check_delay(65535).unwrap(), 65535);
}

#[test]
fn macro_delay_65536_is_rejected() {
    assert!(macro_store::check_delay(65536).is_err());
}

#[test]
fn macro_flag_must_be_down_or_up() {
    assert!(Event::decode(&[0, 0, 0x04, 0x90]).is_err());
    assert!(Event::decode(&[0x0A, 0x00, 0x04, 0xB0]).unwrap().down);
}

#[test]
fn report_rate_7_is_rejected() {
    assert!(Globals::default().set_report_rate(7).is_err());
}

#[test]
fn report_rate_6_lands_on_byte_5() {
    let mut globals = Globals::default();
    globals.set_report_rate(6).unwrap();
    assert_eq!(globals.raw[5], 6);
}

#[test]
fn deadzone_0_51_is_rejected() {
    assert!(Globals::default().set_top_dead(51).is_err());
}

#[test]
fn deadzone_0_50_is_accepted() {
    let mut globals = Globals::default();
    globals.set_top_dead(50).unwrap();
    assert_eq!(globals.raw[8], 50);
}

#[test]
fn sleep_31_minutes_is_rejected() {
    assert!(Globals::default().set_sleep_minutes(31).is_err());
}

#[test]
fn global_edit_preserves_unowned_bytes() {
    let mut globals = Globals::default();
    globals.raw[20] = 0x5A;
    globals.set_report_rate(3).unwrap();
    assert_eq!(globals.raw[20], 0x5A);
    assert_eq!(globals.raw[5], 3);
}

#[test]
fn brightness_6_is_rejected() {
    let light = Lighting {
        brightness: 6,
        ..Lighting::default()
    };
    assert!(light.encode().is_err());
}

#[test]
fn speed_6_is_rejected() {
    let light = Lighting {
        speed: 6,
        ..Lighting::default()
    };
    assert!(light.encode().is_err());
}

#[test]
fn mode_20_is_followed_by_the_color_table() {
    let mut board = wired_board();
    board.lighting.mode = 20;
    let order = board.command_order();
    assert_eq!(order[order.len() - 2], SET_LIGHT);
    assert_eq!(*order.last().unwrap(), SET_COLOR);
    assert_eq!(lighting::encode_colors(&board.colors).len(), 504);
}

#[test]
fn color_record_starts_with_the_key_index() {
    let colors = [[9u8, 8, 7]; lighting::COLOR_RECORDS];
    let bytes = lighting::encode_colors(&colors);
    assert_eq!(bytes[0], 0);
    assert_eq!(bytes[4], 1);
    assert_eq!(bytes[4 * 17], 17);
}

#[test]
fn mode_21_reads_as_off() {
    let mut raw = Lighting::default().encode().unwrap();
    raw[0] = 21;
    assert_eq!(Lighting::decode(&raw).unwrap().mode, 0);
}

#[test]
fn color_mode_zero_is_preserved() {
    let light = Lighting {
        color_mode: 0,
        ..Lighting::default()
    };
    assert_eq!(light.encode().unwrap()[8], 0);
}

#[test]
fn short_fn_pack_is_rejected() {
    let json = r#"{"allFnPack":[1,2,3],"RTlist":[]}"#;
    assert!(profile::parse_document(json).is_err());
}

#[test]
fn rt_list_of_1024_uses_only_the_first_1008() {
    let mut bytes = vec![0u8; 1024];
    bytes[1007] = 0xAB;
    bytes[1023] = 0xFF;
    let list = serde_json::to_string(&bytes).unwrap();
    let json = format!(r#"{{"RTlist":{list}}}"#);
    let board = profile::parse_document(&json).unwrap();
    let table = actuation::encode_table(&board.actuation);
    assert_eq!(table.len(), 1008);
    assert_eq!(table[1007], 0xAB);
}

#[test]
fn hec_profile_without_rt_list_is_rejected() {
    assert!(profile::parse_document(r#"{"allFnPack":[]}"#).is_err());
}

#[test]
fn version_2_is_rejected() {
    let json = r#"{"format":"mini60-config","version":2,"device":{"vid":"0x0C45","pid":"0x80A2","firmware":"1.53"}}"#;
    assert!(profile::parse_document(json).is_err());
}

#[test]
fn another_product_id_is_rejected() {
    let mut board = wired_board();
    board.info = Some(hekit::info::DeviceInfo {
        vid: 0x0C45,
        pid: 0x80A2,
        firmware: "1.53".into(),
        battery: 0,
    });
    let mut text = profile::to_pretty(&board).unwrap();
    text = text.replace("0x80A2", "0x1234");
    assert!(profile::parse_document(&text).is_err());
}

#[test]
fn jade_ultra_profile_writes_axis_byte_2() {
    let mut board = wired_board();
    board.set_switch(17, SwitchChoice::JadeUltra).unwrap();
    board.set_trip(17, 200).unwrap();
    assert_eq!(board.actuation[17].axis_type, 2);
    assert_eq!(board.choice[17], SwitchChoice::JadeUltra);
    let text = profile::to_pretty(&board).unwrap();
    let loaded = profile::parse_document(&text).unwrap();
    assert_eq!(loaded.actuation[17].axis_type, 2);
    assert_eq!(loaded.choice[17], SwitchChoice::JadeUltra);
}

#[test]
fn round_trip_keeps_3_00_1_25_and_0_75() {
    let mut board = wired_board();
    board.info = Some(hekit::info::DeviceInfo {
        vid: 0x0C45,
        pid: 0x80A2,
        firmware: "1.53".into(),
        battery: 0,
    });
    board.set_trip(0, 300).unwrap();
    board.set_rt_press(0, 125).unwrap();
    board.set_rt_release(0, 75).unwrap();
    let loaded = profile::parse_document(&profile::to_pretty(&board).unwrap()).unwrap();
    let raw = loaded.actuation[0].encode();
    assert_eq!(&raw[2..4], &300u16.to_le_bytes());
    assert_eq!(&raw[4..6], &125u16.to_le_bytes());
    assert_eq!(&raw[6..8], &75u16.to_le_bytes());
}

#[test]
fn truncated_json_writes_nothing() {
    assert!(profile::parse_document("{").is_err());
}

#[test]
fn no_path_emits_factory_reset_or_flash() {
    for cmd in [0x0F_u8, 0x4F] {
        assert!(matches!(
            frame::build_request(WIRED_FRAME, cmd, 0, &[], true),
            Err(Error::ForbiddenCommand(_))
        ));
    }
}

#[test]
fn no_path_emits_calibration_wipe() {
    for cmd in [0x64_u8, 0x65] {
        assert!(matches!(
            frame::paginate(WIRED_FRAME, cmd, &[0]),
            Err(Error::ForbiddenCommand(_))
        ));
    }
}

#[test]
fn mouse_page_and_mod_tap_and_toggle_round_trip() {
    assert_eq!(keys::mouse(1, 1).page, 1);
    assert_eq!(keys::mod_tap(0x29, 0xE0, 0x19).encode()[0], 9);
    assert_eq!(keys::toggle(0x2C).encode(), [10, 0x2C, 0, 0]);
}

#[test]
fn write_order_is_globals_actuation_dks_macros_base_fn_light() {
    let board = wired_board();
    assert_eq!(
        board.command_order(),
        vec![
            SET_GLOBALS,
            SET_ACTUATION,
            SET_DKS,
            frame::SET_MACRO,
            SET_KEYS,
            SET_FN,
            SET_LIGHT
        ]
    );
}

#[test]
fn device_info_v153() {
    let mut payload = vec![0u8; 56];
    payload[4] = 0x45;
    payload[5] = 0x0C;
    payload[6] = 0xA2;
    payload[7] = 0x80;
    payload[8] = 0x53;
    payload[9] = 0x01;
    let info = hekit::info::parse(&payload).unwrap();
    assert_eq!(info.vid, 0x0C45);
    assert_eq!(info.pid, 0x80A2);
    assert_eq!(info.firmware, "1.53");
}
