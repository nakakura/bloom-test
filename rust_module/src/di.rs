use shaku::module;

use crate::application::factory::FactoryImpl;
use crate::application::usecase::data::connect::Connect;
use crate::application::usecase::data::redirect::Redirect;
use crate::application::usecase::event;
use crate::application::usecase::general::service::General;
use crate::application::usecase::media::answer::AnswerService;
use crate::application::usecase::media::call::Call;
use crate::application::usecase::peer::create::Create;
use crate::application::usecase::system::System;
use crate::ffi::rust_to_c_bridge::state_objects::{
    CallbackFunctionsImpl, GlobalStateImpl, LoggerImpl, ProgramStateImpl,
};
use crate::infra::RepositoryImpl;

module! {
    pub(crate) CppObjctsModule {
        components = [ProgramStateImpl, CallbackFunctionsImpl, LoggerImpl, GlobalStateImpl],
        providers = []
    }
}

module! {
    pub(crate) GeneralFactory {
        components = [FactoryImpl],
        providers = []
    }
}

module! {
    pub(crate) RepositoryModule {
        components = [RepositoryImpl, GlobalStateImpl],
        providers = []
    }
}

module! {
    pub(crate) GeneralService {
        components = [General, RepositoryImpl, GlobalStateImpl],
        providers = []
    }
}

module! {
    pub(crate) SystemService {
        components = [System],
        providers = []
    }
}

module! {
    pub(crate) PeerCreateService {
        components = [Create, GlobalStateImpl, RepositoryImpl, FactoryImpl, CallbackFunctionsImpl],
        providers = []
    }
}

module! {
    pub(crate) DataConnectService {
        components = [Connect, GlobalStateImpl, RepositoryImpl, FactoryImpl, CallbackFunctionsImpl],
        providers = []
    }
}

module! {
    pub(crate) DataRedirectService {
        components = [Redirect, GlobalStateImpl, RepositoryImpl, FactoryImpl, CallbackFunctionsImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaCallService {
        components = [Call, GlobalStateImpl, RepositoryImpl, FactoryImpl, CallbackFunctionsImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaAnswerService {
        components = [AnswerService, GlobalStateImpl, RepositoryImpl, FactoryImpl, CallbackFunctionsImpl],
        providers = []
    }
}

module! {
    pub(crate) EventReceiveService {
        components = [event::EventReceiveImpl, CallbackFunctionsImpl, GlobalStateImpl, RepositoryImpl, LoggerImpl],
        providers = []
    }
}
