use std::ffi::OsString;
use windows::{
    core::{ComInterface, IUnknown},
    System::DispatcherQueueController,
    Win32::{
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_12_0,
            Direct3D12::{
                D3D12CreateDevice, ID3D12CommandAllocator, ID3D12CommandQueue, ID3D12Device, ID3D12Fence,
                D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC,
            },
            DirectWrite::{DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED},
            Dxgi::{CreateDXGIFactory2, IDXGIAdapter1, IDXGIFactory3, DXGI_ADAPTER_DESC1},
        },
        System::{
            Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
            WinRT::{CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_NONE, DQTYPE_THREAD_CURRENT},
        },
        UI::Input::KeyboardAndMouse::GetDoubleClickTime,
    },
};

fn main() {
    //=========================================================
    // DXGI Factory and adapter enumeration

    // SAFETY: the paramters are valid
    let dxgi_factory = unsafe { CreateDXGIFactory2::<IDXGIFactory3>(0).unwrap() };

    // --- Enumerate adapters
    let mut adapters = Vec::new();
    unsafe {
        let mut i = 0;
        while let Ok(adapter) = dxgi_factory.EnumAdapters1(i) {
            adapters.push(adapter);
            i += 1;
        }
    };

    let mut chosen_adapter = None;
    for adapter in adapters.iter() {
        let mut desc = DXGI_ADAPTER_DESC1::default();
        unsafe { adapter.GetDesc1(&mut desc).unwrap() };

        use std::os::windows::ffi::OsStringExt;

        let name = &desc.Description[..];
        let name_len = name.iter().take_while(|&&c| c != 0).count();
        let name = OsString::from_wide(&desc.Description[..name_len])
            .to_string_lossy()
            .into_owned();
        eprintln!(
                "DXGI adapter: name={}, LUID={:08x}{:08x}",
                name,
                desc.AdapterLuid.HighPart,
                desc.AdapterLuid.LowPart,
            );
        /*if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0) != 0 {
            continue;
        }*/
        if chosen_adapter.is_none() {
            chosen_adapter = Some(adapter.clone())
        }
    }

    //=========================================================
    // D3D12 stuff

    let d3d12_device = unsafe {
        let mut d3d12_device: Option<ID3D12Device> = None;
        let adapter = chosen_adapter
            .as_ref()
            .map(|adapter| adapter.cast::<IUnknown>().unwrap());
        D3D12CreateDevice(
            // pAdapter:
            adapter.as_ref(),
            // MinimumFeatureLevel:
            D3D_FEATURE_LEVEL_12_0,
            // ppDevice:
            &mut d3d12_device,
        )
            .expect("D3D12CreateDevice failed");
        d3d12_device.unwrap()
    };

    //=========================================================
    // WGPU init
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::DX12,
        flags: Default::default(),
        dx12_shader_compiler: Default::default(),
        gles_minor_version: Default::default(),
    });

    unsafe {
        d3d12_device.GetDeviceRemovedReason().expect("device was removed")
    }
}
