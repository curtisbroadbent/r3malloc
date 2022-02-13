use crate::defines::page;
use crate::heap::max_block_num;
use core::assert;

#[derive(Copy, Clone)]
struct SizeClassData {
	block_size: u32,
	sb_size: u32,
	block_num: u32,
	cache_block_num: u32
}

#[inline(always)]
const fn max_sz_idx() -> usize {
	40
}

#[inline(always)]
const fn max_sz() -> usize {
	(1 << 13) + (1 << 11) * 3
}

// FIXME: implement this
#[inline(always)]
const fn size_classes() -> [SizeClassData; max_sz()] {
	[SizeClassData { block_size: 0, sb_size: 0, block_num: 0, cache_block_num: 0 }; max_sz()]
}

static mut SIZE_CLASSES: [SizeClassData; max_sz()] = size_classes();
static mut SIZE_CLASS_LOOKUP: [usize; max_sz() + 1] = [0; max_sz() + 1];

pub fn init_size_class() {
	// each superblock has to contain several block *perfectly*
	for sc_idx in 1..max_sz_idx() {
		let sc = unsafe { SIZE_CLASSES[sc_idx] };
		let block_size = sc.block_size;
		let mut sb_size = sc.sb_size;

		if sb_size > block_size && (sb_size % block_size) == 0 {
			continue;
		}

		while block_size >= sb_size {
			sb_size += sc.sb_size;
		}

		// update value in SIZE_CLASSES
		unsafe { SIZE_CLASSES[sc_idx].sb_size = sc.sb_size; }
	}

	for sc_idx in 1..max_sz_idx() {
		let mut sc = unsafe { SIZE_CLASSES[sc_idx] };
		let mut sb_size = sc.sb_size;

		// increase superblock size if needed
		// 64 KB		
		while sb_size < (16 * page()) as u32 {
			sb_size += sc.sb_size;
		}
		sc.sb_size = sb_size;

		// fill block_num and cache_block_num
		sc.block_num = sc.sb_size / sc.block_size;
		sc.cache_block_num = sc.block_num * 1;

		assert!(sc.block_num > 0);
		assert!(sc.block_num < max_block_num());
		assert!(sc.block_num >= sc.cache_block_num);

		// update value in SIZE_CLASSES
		unsafe { SIZE_CLASSES[sc_idx] = sc; }	
	}

	// first size class reserved for large allocations
	let mut lookup_idx: usize = 0;
	for sc_idx in 1..max_sz_idx() {
		let sc = unsafe { SIZE_CLASSES[sc_idx] };
		while lookup_idx <= sc.block_size as usize {
			unsafe { SIZE_CLASS_LOOKUP[lookup_idx] = sc_idx; }
			lookup_idx += 1;
		}
	}
}