/// ヘッドレスwgpuレンダラーによるスクリーンショット生成
///
/// egui::Context + egui_wgpu::Renderer でオフスクリーンレンダリングし、
/// PNG画像として保存する。ウィンドウ不要でCI環境でも動作する。
use egui::epaint::ClippedPrimitive;
use std::path::Path;
use zoom_video_mover_lib::gui::{setup_gui_appearance, ZoomDownloaderApp};

/// スクリーンショットのサイズ（本番ウィンドウと同じ）
const SCREENSHOT_WIDTH: u32 = 820;
const SCREENSHOT_HEIGHT: u32 = 650;

/// wgpuデバイスとキューをヘッドレスで作成する
fn create_headless_device() -> (wgpu::Device, wgpu::Queue) {
    pollster::block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true,
                compatible_surface: None,
            })
            .await
            .or_else(|| {
                // フォールバック: force_fallback_adapter なしで再試行
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    force_fallback_adapter: false,
                    compatible_surface: None,
                }))
            })
            .expect("Failed to find a suitable GPU adapter");

        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("screenshot_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .expect("Failed to create wgpu device")
    })
}

/// egui UIをオフスクリーンレンダリングしてPNGに保存する
///
/// 事前条件:
/// - app は適切な画面状態に設定済み
/// - output_path の親ディレクトリが存在する（なければ自動作成）
///
/// 事後条件:
/// - 指定パスにPNG画像が保存される
pub fn render_app_to_png(app: &mut ZoomDownloaderApp, output_path: &Path) {
    let (device, queue) = create_headless_device();

    let ctx = egui::Context::default();
    setup_gui_appearance(&ctx);

    // wgpuレンダラーを先に作成（全フレームのテクスチャデルタを適用するため）
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = egui_wgpu::Renderer::new(&device, format, None, 1);

    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(SCREENSHOT_WIDTH as f32, SCREENSHOT_HEIGHT as f32),
        )),
        ..Default::default()
    };

    // ウォームアップフレーム（フォントアトラス生成 + レイアウト安定化）
    // eguiは最初のフレームでフォントテクスチャを生成し、2回目以降は送信しない。
    // 全フレームのテクスチャデルタをレンダラーに適用する必要がある。
    for _ in 0..2 {
        let warmup_output = ctx.run(raw_input.clone(), |ctx| {
            app.update_ui(ctx);
        });
        for (id, delta) in &warmup_output.textures_delta.set {
            renderer.update_texture(&device, &queue, *id, delta);
        }
        for id in &warmup_output.textures_delta.free {
            renderer.free_texture(id);
        }
    }

    // 本番フレーム実行
    let full_output = ctx.run(raw_input, |ctx| {
        app.update_ui(ctx);
    });

    // 本番フレームのテクスチャデルタも適用
    for (id, delta) in &full_output.textures_delta.set {
        renderer.update_texture(&device, &queue, *id, delta);
    }

    let clipped_primitives: Vec<ClippedPrimitive> =
        ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

    // オフスクリーンテクスチャを作成
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("screenshot_texture"),
        size: wgpu::Extent3d {
            width: SCREENSHOT_WIDTH,
            height: SCREENSHOT_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // バッファを用意（テクスチャ→CPUコピー用）
    let bytes_per_row = 4 * SCREENSHOT_WIDTH;
    let aligned_bytes_per_row = (bytes_per_row + 255) & !255; // 256バイトアラインメント
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("screenshot_buffer"),
        size: (aligned_bytes_per_row * SCREENSHOT_HEIGHT) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // レンダリングコマンド作成
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("screenshot_encoder"),
    });

    // egui描画データのGPUバッファアップロード
    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [SCREENSHOT_WIDTH, SCREENSHOT_HEIGHT],
        pixels_per_point: full_output.pixels_per_point,
    };

    renderer.update_buffers(
        &device,
        &queue,
        &mut encoder,
        &clipped_primitives,
        &screen_descriptor,
    );

    // レンダーパスで描画
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("screenshot_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.97,
                        g: 0.97,
                        b: 0.97,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        renderer.render(&mut render_pass, &clipped_primitives, &screen_descriptor);
    }

    // テクスチャ→バッファコピー
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(aligned_bytes_per_row),
                rows_per_image: Some(SCREENSHOT_HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: SCREENSHOT_WIDTH,
            height: SCREENSHOT_HEIGHT,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(std::iter::once(encoder.finish()));

    // バッファからCPUに読み取り
    let buffer_slice = buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);
    receiver.recv().unwrap().expect("Failed to map buffer");

    let data = buffer_slice.get_mapped_range();

    // アラインメント分のパディングを除去してRGBA画像を構築
    let mut img_data = Vec::with_capacity((4 * SCREENSHOT_WIDTH * SCREENSHOT_HEIGHT) as usize);
    for row in 0..SCREENSHOT_HEIGHT {
        let start = (row * aligned_bytes_per_row) as usize;
        let end = start + (4 * SCREENSHOT_WIDTH) as usize;
        img_data.extend_from_slice(&data[start..end]);
    }

    drop(data);
    buffer.unmap();

    // テクスチャデルタのクリーンアップ
    for id in &full_output.textures_delta.free {
        renderer.free_texture(id);
    }

    // PNG保存
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create snapshot directory");
    }

    let img: image::RgbaImage =
        image::ImageBuffer::from_raw(SCREENSHOT_WIDTH, SCREENSHOT_HEIGHT, img_data)
            .expect("Failed to create image buffer");
    img.save(output_path)
        .expect("Failed to save screenshot PNG");

    println!("Screenshot saved: {}", output_path.display());
}
