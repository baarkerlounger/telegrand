use gtk::glib;

use crate::session::chat::Chat;
use crate::session::User;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "MessageForwardOrigin")]
pub(crate) enum MessageForwardOrigin {
    User(User),
    Chat {
        // author_signature: String,
        chat: Chat,
    },
    Channel {
        // author_signature: String,
        chat: Chat,
        // Using a WeakRef here as messages can be deleted.
        // message: WeakRef<Message>,
    },
    HiddenUser {
        sender_name: String,
    },
    MessageImport {
        sender_name: String,
    },
}

impl MessageForwardOrigin {
    pub(crate) fn id(&self) -> Option<i64> {
        Some(match self {
            Self::User(user) => user.id(),
            Self::Chat { chat, .. } | Self::Channel { chat, .. } => chat.id(),
            _ => return None,
        })
    }
}

#[gobject::class(final)]
mod message_forward_info {
    use super::*;
    use gtk::subclass::prelude::*;
    use once_cell::unsync::OnceCell;
    use std::cell::Cell;
    use tdlib::enums::MessageForwardOrigin as TdMessageForwardOrigin;
    use tdlib::types::MessageForwardInfo as TdMessageForwardInfo;

    #[derive(Default)]
    pub(crate) struct MessageForwardInfo {
        #[property(get, boxed)]
        origin: OnceCell<MessageForwardOrigin>,
        #[property(get)]
        date: Cell<i32>,
    }

    impl super::MessageForwardInfo {
        pub(crate) fn from_td_object(td_object: TdMessageForwardInfo, chat: &Chat) -> Self {
            let obj: Self = glib::Object::new(&[]).unwrap();
            let imp = obj.imp();

            let origin = match td_object.origin {
                TdMessageForwardOrigin::User(data) => {
                    MessageForwardOrigin::User(chat.session().user_list().get(data.sender_user_id))
                }
                TdMessageForwardOrigin::Chat(data) => MessageForwardOrigin::Chat {
                    // author_signature: data.author_signature,
                    chat: chat.session().chat_list().get(data.sender_chat_id),
                },
                TdMessageForwardOrigin::Channel(data) => {
                    let chat = chat.session().chat_list().get(data.chat_id);
                    // let message = {
                    //     let weak = WeakRef::new();
                    //     weak.set(chat.history().message_by_id(data.message_id).as_ref());
                    //     weak
                    // };
                    MessageForwardOrigin::Channel {
                        // author_signature: data.author_signature,
                        chat,
                        // message,
                    }
                }
                TdMessageForwardOrigin::HiddenUser(data) => MessageForwardOrigin::HiddenUser {
                    sender_name: data.sender_name,
                },
                TdMessageForwardOrigin::MessageImport(data) => {
                    MessageForwardOrigin::MessageImport {
                        sender_name: data.sender_name,
                    }
                }
            };

            imp.origin.set(origin).unwrap();
            imp.date.set(td_object.date);

            obj
        }
    }
}
