pub enum NumericSlotMut<'a> {
    F32(&'a mut f32),
    F64(&'a mut f64),
    U8(&'a mut u8),
    U16(&'a mut u16),
    U32(&'a mut u32),
    U64(&'a mut u64),
    I8(&'a mut i8),
    I16(&'a mut i16),
    I32(&'a mut i32),
    I64(&'a mut i64),
    I128(&'a mut i128),
}

// TODO: Do we need these functions?
#[allow(dead_code)]
#[inline(always)]
pub(crate) fn apply_numeric_slot_mut(
    slot: NumericSlotMut<'_>,
    mut fn_f32: impl FnMut(f32) -> f32,
    mut fn_f64: impl FnMut(f64) -> f64,
    mut fn_int: impl FnMut(i128, bool) -> i128,
) {
    match slot {
        NumericSlotMut::F32(v) => *v = fn_f32(*v),
        NumericSlotMut::F64(v) => *v = fn_f64(*v),
        NumericSlotMut::U8(v) => *v = fn_int(*v as i128, true).max(0).min(u8::MAX as i128) as u8,
        NumericSlotMut::U16(v) => *v = fn_int(*v as i128, true).max(0).min(u16::MAX as i128) as u16,
        NumericSlotMut::U32(v) => *v = fn_int(*v as i128, true).max(0).min(u32::MAX as i128) as u32,
        NumericSlotMut::U64(v) => *v = fn_int(*v as i128, true).max(0).min(u64::MAX as i128) as u64,
        NumericSlotMut::I8(v) => {
            *v = fn_int(*v as i128, false).clamp(i8::MIN as i128, i8::MAX as i128) as i8
        }
        NumericSlotMut::I16(v) => {
            *v = fn_int(*v as i128, false).clamp(i16::MIN as i128, i16::MAX as i128) as i16
        }
        NumericSlotMut::I32(v) => {
            *v = fn_int(*v as i128, false).clamp(i32::MIN as i128, i32::MAX as i128) as i32
        }
        NumericSlotMut::I64(v) => {
            *v = fn_int(*v as i128, false).clamp(i64::MIN as i128, i64::MAX as i128) as i64
        }
        NumericSlotMut::I128(v) => *v = fn_int(*v as i128, false),
    }
}

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn apply_pair_numeric_slot_mut(
    slot_one: NumericSlotMut<'_>,
    slot_two: NumericSlotMut<'_>,
    mut fn_f32: impl FnMut(f32, f32) -> (f32, f32),
    mut fn_f64: impl FnMut(f64, f64) -> (f64, f64),
    mut fn_int: impl FnMut(i128, i128, bool) -> (i128, i128),
) {
    match (slot_one, slot_two) {
        (NumericSlotMut::F32(v1), NumericSlotMut::F32(v2)) => {
            let (new_v1, new_v2) = fn_f32(*v1, *v2);
            *v1 = new_v1;
            *v2 = new_v2;
        }
        (NumericSlotMut::F64(v1), NumericSlotMut::F64(v2)) => {
            let (new_v1, new_v2) = fn_f64(*v1, *v2);
            *v1 = new_v1;
            *v2 = new_v2;
        }
        (NumericSlotMut::U8(v1), NumericSlotMut::U8(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, true);
            *v1 = new_v1.max(0).min(u8::MAX as i128) as u8;
            *v2 = new_v2.max(0).min(u8::MAX as i128) as u8;
        }
        (NumericSlotMut::U16(v1), NumericSlotMut::U16(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, true);
            *v1 = new_v1.max(0).min(u16::MAX as i128) as u16;
            *v2 = new_v2.max(0).min(u16::MAX as i128) as u16;
        }
        (NumericSlotMut::U32(v1), NumericSlotMut::U32(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, true);
            *v1 = new_v1.max(0).min(u32::MAX as i128) as u32;
            *v2 = new_v2.max(0).min(u32::MAX as i128) as u32;
        }
        (NumericSlotMut::U64(v1), NumericSlotMut::U64(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, true);
            *v1 = new_v1.max(0).min(u64::MAX as i128) as u64;
            *v2 = new_v2.max(0).min(u64::MAX as i128) as u64;
        }
        (NumericSlotMut::I8(v1), NumericSlotMut::I8(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1.clamp(i8::MIN as i128, i8::MAX as i128) as i8;
            *v2 = new_v2.clamp(i8::MIN as i128, i8::MAX as i128) as i8;
        }
        (NumericSlotMut::I16(v1), NumericSlotMut::I16(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1.clamp(i16::MIN as i128, i16::MAX as i128) as i16;
            *v2 = new_v2.clamp(i16::MIN as i128, i16::MAX as i128) as i16;
        }
        (NumericSlotMut::I32(v1), NumericSlotMut::I32(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1.clamp(i32::MIN as i128, i32::MAX as i128) as i32;
            *v2 = new_v2.clamp(i32::MIN as i128, i32::MAX as i128) as i32;
        }
        (NumericSlotMut::I64(v1), NumericSlotMut::I64(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
            *v2 = new_v2.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        }
        (NumericSlotMut::I128(v1), NumericSlotMut::I128(v2)) => {
            let (new_v1, new_v2) = fn_int(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1;
            *v2 = new_v2;
        }
        _ => {}
    }
}
