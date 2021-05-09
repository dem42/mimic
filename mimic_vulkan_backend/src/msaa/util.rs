use ash::vk;
use log::info;

pub fn get_max_sample_count(
    physical_device_properties: vk::PhysicalDeviceProperties,
) -> vk::SampleCountFlags {
    // get the sample counts (represented by bits) which the color and depth framebuffer have in common
    let counts = physical_device_properties
        .limits
        .framebuffer_color_sample_counts
        & physical_device_properties
            .limits
            .framebuffer_depth_sample_counts;
    let flags = [
        vk::SampleCountFlags::TYPE_64,
        vk::SampleCountFlags::TYPE_32,
        vk::SampleCountFlags::TYPE_16,
        vk::SampleCountFlags::TYPE_8,
        vk::SampleCountFlags::TYPE_4,
        vk::SampleCountFlags::TYPE_2,
    ];
    for &flag in flags.iter() {
        if counts.contains(flag) {
            info!("MSAA max samples count is {:?}", flag);
            return flag;
        }
    }

    info!(
        "MSAA max samples count is {:?}",
        vk::SampleCountFlags::TYPE_1
    );
    vk::SampleCountFlags::TYPE_1
}
