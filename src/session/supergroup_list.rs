#[gobject::class(final, implements(gtk::gio::ListModel))]
mod supergroup_list {
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use indexmap::map::Entry;
    use indexmap::IndexMap;
    use std::cell::RefCell;
    use tdlib::enums::Update;

    use crate::session::Supergroup;

    #[derive(Default)]
    pub(crate) struct SupergroupList {
        list: RefCell<IndexMap<i64, Supergroup>>,
    }

    impl ListModelImpl for SupergroupList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Supergroup::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get_index(position as usize)
                .map(|(_, o)| o.clone().upcast::<glib::Object>())
        }
    }

    impl super::SupergroupList {
        pub(crate) fn new() -> Self {
            glib::Object::new(&[]).unwrap()
        }

        /// Return the `Supergroup` of the specified `id`. Panics if the supergroup is not present.
        /// Note that TDLib guarantees that objects are always returned before of their ids,
        /// so if you use an `id` returned by TDLib, it should be expected that the relative
        /// `Supergroup` exists in the list.
        pub(crate) fn get(&self, id: i64) -> Supergroup {
            self.imp().list.borrow().get(&id).unwrap().clone()
        }

        pub(crate) fn handle_update(&self, update: Update) {
            if let Update::Supergroup(data) = update {
                let mut list = self.imp().list.borrow_mut();

                match list.entry(data.supergroup.id) {
                    Entry::Occupied(entry) => entry.get().handle_update(Update::Supergroup(data)),
                    Entry::Vacant(entry) => {
                        let supergroup = data.supergroup.into();
                        entry.insert(supergroup);

                        let index = list.len() - 1;
                        drop(list);

                        self.items_changed(index as u32, 0, 1);
                    }
                }
            }
        }
    }
}
