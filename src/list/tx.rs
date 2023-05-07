use std::collections::{HashMap, hash_map::{Entry, VacantEntry}};
use super::{List, ItemMut};

pub struct TxItemMut<'a, L: List + 'a> {
    kind: TxItemMutKind<'a, L>,
}
enum TxItemMutKind<'a, L: List + 'a> {
    Original(L::ItemMut<'a>, VacantEntry<'a, usize, L::Item>),
    ReplacedOrPushed(&'a mut L::Item),
}

impl<'a, L: List> ItemMut<'a, L::Item> for TxItemMut<'a, L> {
    fn set(&mut self, item: L::Item) {
        ::take_mut::take(&mut self.kind, |kind| {
            match kind {
                TxItemMutKind::Original(_, vacant) => TxItemMutKind::ReplacedOrPushed(vacant.insert(item)),
                TxItemMutKind::ReplacedOrPushed(item_mut_ref) => {
                    *item_mut_ref = item;
                    TxItemMutKind::ReplacedOrPushed(item_mut_ref)
                }
            }
        });
    }
    fn get(&self) -> &L::Item {
        match &self.kind {
            TxItemMutKind::Original(base_item_mut, _) => base_item_mut.get(),
            TxItemMutKind::ReplacedOrPushed(item) => *item,
        }
    }
}

pub struct TxList<L: List> {
    base: L,
    replaced_items: HashMap<usize, L::Item>,
    pushed_items: Vec<L::Item>,
}

impl<L: List> List for TxList<L> {
    type Item = L::Item;
    type ItemMut<'a> where Self: 'a = TxItemMut<'a, L>;

    fn len(&self) -> usize {
        self.base.len() + self.pushed_items.len()
    }
    fn push(&mut self, item: Self::Item) {
        self.pushed_items.push(item);
    }
    fn get(&self, idx: usize) -> Option<&Self::Item> {
        if let Some(idx_in_pushed_items) = idx.checked_sub(self.base.len()) {
            self.pushed_items.get(idx_in_pushed_items)
        }
        else if let Some(replaced_item) = self.replaced_items.get(&idx) {
            Some(replaced_item)
        } else {
            self.base.get(idx)
        }
    }
    fn get_mut(&mut self, idx: usize) -> Option<Self::ItemMut<'_>> {
        if let Some(idx_in_pushed_items) = idx.checked_sub(self.base.len()) {
            let pushed_item = self.pushed_items.as_mut_slice().get_mut(idx_in_pushed_items)?;
            Some(TxItemMut {
                kind: TxItemMutKind::ReplacedOrPushed(pushed_item),
            })
        } else {
            match self.replaced_items.entry(idx) {
                Entry::Occupied(occupied) => Some(TxItemMut {
                    kind: TxItemMutKind::ReplacedOrPushed(occupied.into_mut()),
                }),
                Entry::Vacant(vacant) => {
                    let original_ref = self.base.get_mut(idx)?;
                    Some(TxItemMut {
                        kind: TxItemMutKind::Original(original_ref, vacant),
                    })
                }
            }
        }
    }
}
