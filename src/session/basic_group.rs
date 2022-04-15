#[gobject::class(final)]
mod basic_group {
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::Cell;
    use tdlib::enums::Update;
    use tdlib::types::BasicGroup as TdBasicGroup;

    #[derive(Default)]
    pub(crate) struct BasicGroup {
        #[property(get)]
        id: Cell<i64>,
        #[property(get)]
        member_count: Cell<i32>,
    }

    impl super::BasicGroup {
        pub(crate) fn handle_update(&self, update: Update) {
            if let Update::BasicGroup(data) = update {
                self.set_member_count(data.basic_group.member_count);
            }
        }

        fn set_member_count(&self, member_count: i32) {
            let imp = self.imp();
            let old = imp.member_count.replace(member_count);
            if old != imp.member_count.get() {
                self.notify_member_count();
            }
        }
    }

    impl From<TdBasicGroup> for super::BasicGroup {
        fn from(td_object: TdBasicGroup) -> Self {
            let obj: Self = glib::Object::new(&[]).unwrap();
            let imp = obj.imp();

            imp.id.set(td_object.id);
            imp.member_count.set(td_object.member_count);

            obj
        }
    }
}
