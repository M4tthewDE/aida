use chrono::Utc;
use ipc_channel::ipc::IpcSender;
use std::{ffi::CStr, os::raw::c_int, path::PathBuf, sync::OnceLock};
use tracing::debug;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[allow(warnings)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

static SENDER: OnceLock<IpcSender<shared::AgentMessage>> = OnceLock::new();
static CONFIG: OnceLock<shared::Config> = OnceLock::new();

#[unsafe(export_name = "Agent_OnLoad")]
extern "C" fn agent_on_load(
    jvm: *mut bindings::JavaVM,
    options: *mut i8,
    _reserved: *mut std::ffi::c_void,
) -> c_int {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()
        .ok();

    unsafe {
        let options = CStr::from_ptr(options).to_str().unwrap();
        let mut options = options.split(",");
        let server_name = options.next().unwrap();
        let config_arg = options.next().unwrap();
        let config_path = PathBuf::from(config_arg);
        let config = shared::load_config(config_path);
        CONFIG.set(config).unwrap();

        let tx: IpcSender<shared::AgentMessage> =
            IpcSender::connect(server_name.to_string()).unwrap();
        SENDER.set(tx).unwrap();

        let get_env = (*(*jvm)).GetEnv.unwrap();
        let mut env: *mut std::ffi::c_void = std::ptr::null_mut();
        let result = get_env(
            jvm,
            &mut env as *mut *mut std::ffi::c_void,
            bindings::JVMTI_VERSION_1_0 as i32,
        );

        let env = if result == bindings::JNI_OK as i32 {
            env as *mut bindings::jvmtiEnv
        } else {
            panic!("error getting env: {}", result);
        };

        let callbacks = bindings::jvmtiEventCallbacks {
            ClassLoad: Some(class_load),
            MethodEntry: Some(method_entry),
            MethodExit: Some(method_exit),
            ..Default::default()
        };

        let result = (*(*env)).SetEventCallbacks.unwrap()(
            env,
            &callbacks as *const bindings::jvmtiEventCallbacks,
            size_of::<bindings::jvmtiEventCallbacks>() as i32,
        );

        assert_eq!(result, 0);

        let mut capabilities: bindings::jvmtiCapabilities = std::mem::zeroed();
        capabilities.set_can_generate_method_entry_events(1);
        capabilities.set_can_generate_method_exit_events(1);

        let result = (*(*env)).AddCapabilities.unwrap()(env, &capabilities);
        assert_eq!(result, 0);

        let result = (*(*env)).SetEventNotificationMode.unwrap()(
            env,
            bindings::jvmtiEventMode_JVMTI_ENABLE,
            bindings::jvmtiEvent_JVMTI_EVENT_VM_INIT,
            std::ptr::null_mut(),
        );

        assert_eq!(result, 0);

        let result = (*(*env)).SetEventNotificationMode.unwrap()(
            env,
            bindings::jvmtiEventMode_JVMTI_ENABLE,
            bindings::jvmtiEvent_JVMTI_EVENT_CLASS_LOAD,
            std::ptr::null_mut(),
        );

        assert_eq!(result, 0);

        let result = (*(*env)).SetEventNotificationMode.unwrap()(
            env,
            bindings::jvmtiEventMode_JVMTI_ENABLE,
            bindings::jvmtiEvent_JVMTI_EVENT_METHOD_ENTRY,
            std::ptr::null_mut(),
        );

        assert_eq!(result, 0);

        let result = (*(*env)).SetEventNotificationMode.unwrap()(
            env,
            bindings::jvmtiEventMode_JVMTI_ENABLE,
            bindings::jvmtiEvent_JVMTI_EVENT_METHOD_EXIT,
            std::ptr::null_mut(),
        );

        assert_eq!(result, 0);
    }

    debug!("agent loaded");

    0
}

#[unsafe(no_mangle)]
extern "C" fn class_load(
    jvmti_env: *mut bindings::jvmtiEnv,
    _env: *mut bindings::JNIEnv,
    _jthread: bindings::jthread,
    class: bindings::jclass,
) {
    let mut signature: *mut i8 = std::ptr::null_mut();

    unsafe {
        (*(*jvmti_env)).GetClassSignature.unwrap()(
            jvmti_env,
            class,
            &mut signature,
            &mut std::ptr::null_mut(),
        );

        if !signature.is_null() {
            let timestamp = Utc::now().timestamp_micros();
            let signature = CStr::from_ptr(signature).to_string_lossy().to_string();
            let name = signature
                .strip_prefix("L")
                .unwrap()
                .strip_suffix(";")
                .unwrap()
                .replace("/", ".");

            if !CONFIG.get().unwrap().class_loads.contains(&name) {
                return;
            }

            SENDER
                .get()
                .unwrap()
                .send(shared::AgentMessage::ClassLoad(shared::ClassLoadEvent {
                    timestamp,
                    name,
                }))
                .unwrap();
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn method_entry(
    jvmti_env: *mut bindings::jvmtiEnv,
    _env: *mut bindings::JNIEnv,
    _jthread: bindings::jthread,
    jmethod_id: bindings::jmethodID,
) {
    let mut name: *mut i8 = std::ptr::null_mut();

    unsafe {
        (*(*jvmti_env)).GetMethodName.unwrap()(
            jvmti_env,
            jmethod_id,
            &mut name,
            &mut std::ptr::null_mut(),
            &mut std::ptr::null_mut(),
        );
        let name = CStr::from_ptr(name).to_string_lossy().to_string();

        if !CONFIG.get().unwrap().methods.contains(&name) {
            return;
        }

        let mut class: bindings::jclass = std::ptr::null_mut();

        (*(*jvmti_env)).GetMethodDeclaringClass.unwrap()(jvmti_env, jmethod_id, &mut class);

        let mut signature: *mut i8 = std::ptr::null_mut();
        (*(*jvmti_env)).GetClassSignature.unwrap()(
            jvmti_env,
            class,
            &mut signature,
            &mut std::ptr::null_mut(),
        );

        let signature = CStr::from_ptr(signature).to_string_lossy().to_string();
        let class_name = signature
            .strip_prefix("L")
            .unwrap()
            .strip_suffix(";")
            .unwrap()
            .replace("/", ".");

        let timestamp = Utc::now().timestamp_micros();
        SENDER
            .get()
            .unwrap()
            .send(shared::AgentMessage::MethodEvent(
                shared::MethodEvent::Entry {
                    timestamp,
                    name,
                    class_name,
                },
            ))
            .unwrap();
    }
}

#[unsafe(no_mangle)]
extern "C" fn method_exit(
    jvmti_env: *mut bindings::jvmtiEnv,
    _env: *mut bindings::JNIEnv,
    _jthread: bindings::jthread,
    jmethod_id: bindings::jmethodID,
    _was_popped_by_exception: bindings::jboolean,
    _return_value: bindings::jvalue,
) {
    let mut name: *mut i8 = std::ptr::null_mut();

    unsafe {
        (*(*jvmti_env)).GetMethodName.unwrap()(
            jvmti_env,
            jmethod_id,
            &mut name,
            &mut std::ptr::null_mut(),
            &mut std::ptr::null_mut(),
        );

        let name = CStr::from_ptr(name).to_string_lossy().to_string();

        if !CONFIG.get().unwrap().methods.contains(&name) {
            return;
        }

        let mut class: bindings::jclass = std::ptr::null_mut();

        (*(*jvmti_env)).GetMethodDeclaringClass.unwrap()(jvmti_env, jmethod_id, &mut class);

        let mut signature: *mut i8 = std::ptr::null_mut();
        (*(*jvmti_env)).GetClassSignature.unwrap()(
            jvmti_env,
            class,
            &mut signature,
            &mut std::ptr::null_mut(),
        );

        let signature = CStr::from_ptr(signature).to_string_lossy().to_string();
        let class_name = signature
            .strip_prefix("L")
            .unwrap()
            .strip_suffix(";")
            .unwrap()
            .replace("/", ".");

        let timestamp = Utc::now().timestamp_micros();
        SENDER
            .get()
            .unwrap()
            .send(shared::AgentMessage::MethodEvent(
                shared::MethodEvent::Exit {
                    timestamp,
                    name,
                    class_name,
                },
            ))
            .unwrap();
    }
}

#[unsafe(export_name = "Agent_OnUnload")]
pub extern "C" fn agent_on_unload(_vm: *mut bindings::JavaVM) {
    debug!("agent unloaded");
    SENDER
        .get()
        .unwrap()
        .send(shared::AgentMessage::Unload)
        .unwrap();
}
