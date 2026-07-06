function render(scene) {
    console.log(scene);
    
    const canvas = document.getElementById("scene-canvas");
    const ctx = canvas.getContext("2d");

    const dpr = window.devicePixelRatio || 1;
    const targetW = Math.round(scene.width * dpr);
    const targetH = Math.round(scene.height * dpr);
    if (canvas.width !== targetW || canvas.height !== targetH) {{
        canvas.width = targetW;
        canvas.height = targetH;
    }}
    // All drawing below stays in CSS-pixel coordinates; this transform
    // maps them onto the (possibly higher-res) backing store.
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    ctx.clearRect(0, 0, scene.width, scene.height);

    for (const l of scene.lines) {
        ctx.lineWidth = l.width;
        ctx.strokeStyle = l.color;
        ctx.beginPath();
        ctx.moveTo(l.x1, l.y1);
        ctx.lineTo(l.x2, l.y2);
        ctx.stroke();
    }

    ctx.fillStyle = "#f97316";

    for (const [x, y] of scene.points) {
        ctx.beginPath();
        ctx.arc(x, y, 4.0, 0, Math.PI * 2);
        ctx.fill();
    }

    ctx.lineWidth = 2.0;
    for (const [idx, col] of scene.highlight_points) {
        console.log("Highlighting point");
        ctx.strokeStyle = col;
        ctx.beginPath();
        ctx.arc(scene.points[idx][0], scene.points[idx][1], 7.0, 0, Math.PI * 2);
        ctx.stroke();
    }

    console.log("Rendering complete!");
}
