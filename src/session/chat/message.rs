use gtk::glib;
use tdlib::enums::MessageSender as TdMessageSender;

use crate::session::{Chat, Session, User};

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "MessageSender")]
pub(crate) enum MessageSender {
    User(User),
    Chat(Chat),
}

impl MessageSender {
    pub(crate) fn from_td_object(sender: &TdMessageSender, session: &Session) -> Self {
        match sender {
            TdMessageSender::User(data) => {
                let user = session.user_list().get(data.user_id);
                MessageSender::User(user)
            }
            TdMessageSender::Chat(data) => {
                let chat = session.chat_list().get(data.chat_id);
                MessageSender::Chat(chat)
            }
        }
    }

    pub(crate) fn as_user(&self) -> Option<&User> {
        match self {
            MessageSender::User(user) => Some(user),
            _ => None,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        match self {
            Self::User(user) => user.id(),
            Self::Chat(chat) => chat.id(),
        }
    }
}

#[gobject::class(final)]
mod message {
    use super::*;
    use gobject::{ConstructCell, WeakCell};
    use gtk::subclass::prelude::*;
    use once_cell::unsync::OnceCell;
    use std::cell::{Cell, RefCell};
    use tdlib::enums::Update;
    use tdlib::types::Message as TdMessage;

    use crate::expressions;
    use crate::session::chat::{BoxedMessageContent, MessageForwardInfo, MessageForwardOrigin};

    #[derive(Default)]
    pub(crate) struct Message {
        #[property(get)]
        id: Cell<i64>,
        #[property(get, boxed)]
        sender: OnceCell<MessageSender>,
        #[property(get, object)]
        chat: WeakCell<Chat>,
        #[property(get)]
        is_outgoing: Cell<bool>,
        #[property(get)]
        date: Cell<i32>,
        #[property(get, object)]
        forward_info: RefCell<Option<MessageForwardInfo>>,
        #[property(get, boxed)]
        content: ConstructCell<BoxedMessageContent>,
    }

    impl super::Message {
        pub(crate) fn from_td_object(td_object: TdMessage, chat: &Chat) -> Self {
            let obj: Self = glib::Object::new(&[]).unwrap();
            let imp = obj.imp();

            imp.id.set(td_object.id);
            imp.sender
                .set(MessageSender::from_td_object(
                    &td_object.sender_id,
                    &chat.session(),
                ))
                .unwrap();
            imp.chat.set(Some(chat));
            imp.is_outgoing.set(td_object.is_outgoing);
            imp.date.set(td_object.date);
            imp.forward_info.replace(
                td_object
                    .forward_info
                    .map(|f| MessageForwardInfo::from_td_object(f, chat)),
            );
            imp.content
                .replace(Some(BoxedMessageContent(td_object.content)));

            obj
        }

        pub(crate) fn handle_update(&self, update: Update) {
            if let Update::MessageContent(data) = update {
                self.set_content(BoxedMessageContent(data.new_content));
            }
        }

        // TODO: This should be moved to the expressions module
        pub(crate) fn sender_name_expression(&self) -> gtk::Expression {
            match self.sender() {
                MessageSender::User(user) => {
                    let user_expression = gtk::ConstantExpression::new(&user);
                    expressions::user_full_name(&user_expression)
                }
                MessageSender::Chat(chat) => gtk::ConstantExpression::new(&chat)
                    .chain_property::<Chat>("title")
                    .upcast(),
            }
        }

        // TODO: This should be moved to the expressions module
        pub(crate) fn sender_display_name_expression(&self) -> gtk::Expression {
            if self.chat().is_own_chat() {
                self.forward_info()
                    .as_ref()
                    .map(MessageForwardInfo::origin)
                    .map(|forward_origin| match forward_origin {
                        MessageForwardOrigin::User(user) => {
                            let user_expression = gtk::ObjectExpression::new(&user);
                            expressions::user_full_name(&user_expression)
                        }
                        MessageForwardOrigin::Chat { chat, .. }
                        | MessageForwardOrigin::Channel { chat, .. } => {
                            gtk::ConstantExpression::new(&chat)
                                .chain_property::<Chat>("title")
                                .upcast()
                        }
                        MessageForwardOrigin::HiddenUser { sender_name }
                        | MessageForwardOrigin::MessageImport { sender_name } => {
                            gtk::ConstantExpression::new(&sender_name).upcast()
                        }
                    })
                    .unwrap_or_else(|| self.sender_display_name_expression())
            } else {
                self.sender_name_expression()
            }
        }

        fn set_content(&self, content: BoxedMessageContent) {
            let imp = self.imp();
            let old = imp.content.replace(Some(content));
            if old != *imp.content.borrow() {
                self.notify_content();
            }
        }
    }
}
