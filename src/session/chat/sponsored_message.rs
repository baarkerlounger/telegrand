#[gobject::class(final)]
mod sponsored_message {
    use gobject::WeakCell;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use once_cell::unsync::OnceCell;
    use std::cell::Cell;
    use tdlib::types::Error as TdError;
    use tdlib::{enums, functions};

    use crate::session::chat::BoxedMessageContent;
    use crate::session::Chat;
    use crate::Session;

    #[derive(Default)]
    pub(crate) struct SponsoredMessage {
        #[property(get)]
        message_id: Cell<i64>,
        #[property(get, object)]
        sponsor_chat: WeakCell<Chat>,
        #[property(get, boxed)]
        content: OnceCell<BoxedMessageContent>,
    }

    impl super::SponsoredMessage {
        pub(crate) async fn request(chat_id: i64, session: &Session) -> Result<Self, TdError> {
            let enums::SponsoredMessage::SponsoredMessage(td_object) =
                functions::get_chat_sponsored_message(chat_id, session.client_id()).await?;
            let obj: Self = glib::Object::new(&[]).unwrap();
            let imp = obj.imp();

            imp.message_id.set(td_object.message_id);
            imp.sponsor_chat
                .set(Some(&session.chat_list().get(td_object.sponsor_chat_id)));
            imp.content
                .set(BoxedMessageContent(td_object.content))
                .unwrap();

            Ok(obj)
        }
    }
}
