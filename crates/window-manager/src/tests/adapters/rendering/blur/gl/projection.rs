use super::*;

#[test]
fn test_project_rect_to_gl_normal() {
    // Identity projection for 1920x1080 display
    // M00=2/1920, M11=2/1080, M02=-1, M12=-1 (Y up)
    let mut proj = [0.0f32; 9];
    proj[0] = 2.0 / 1920.0;
    proj[4] = 2.0 / 1080.0;
    proj[6] = -1.0;
    proj[7] = -1.0;
    proj[8] = 1.0;

    let dst = Rectangle::new((100, 200).into(), (400, 300).into());
    let (x, y, w, h) = project_rect_to_gl(dst, &proj, 1920, 1080);
    assert_eq!((x, y, w, h), (100, 200, 400, 300));
}

#[test]
fn test_project_rect_to_gl_rotated_90() {
    // Rotated projection for 1600x2560 hardware panel displayed at 2560x1600 (Transform::_270 render transform)
    // With gx = 1600 - ly, gy = lx
    let mut proj = [0.0f32; 9];
    // ndc_x = 1.0 - 2.0 * y / 1600.0 => proj[3] = -2.0/1600.0, proj[6] = 1.0 (row 0)
    proj[3] = -2.0 / 1600.0;
    proj[6] = 1.0;
    // ndc_y = 2.0 * x / 2560.0 - 1.0 => proj[1] = 2.0/2560.0, proj[7] = -1.0 (row 1)
    proj[1] = 2.0 / 2560.0;
    proj[7] = -1.0;
    proj[8] = 1.0;

    // Window at top-left: x=0, y=0, w=400, h=300
    let dst = Rectangle::new((0, 0).into(), (400, 300).into());
    let (x, y, w, h) = project_rect_to_gl(dst, &proj, 1600, 2560);
    // gx range = [1600 - 300, 1600] = [1300, 1600]
    // gy range = [0, 400]
    assert_eq!((x, y, w, h), (1300, 0, 300, 400));
}
