import init, { emoji_filter } from './rust_videoemoji.js';

const emoji_chk = document.getElementById('emoji_chk');
const video = document.getElementById('myVideo');

const normal_canvas = document.getElementById('normalCanvas');
const normal_ctx = normal_canvas.getContext('2d',{willReadFrequently: true,});

navigator.mediaDevices.getUserMedia({ video: true, audio: false })
    .then(stream => {
        video.srcObject = stream;
        video.play();
        // 描画を開始
        video.addEventListener('loadeddata', () => {
            //　const dpr = window.devicePixelRatio || 1;
            normal_canvas.width = video.videoWidth;
            normal_canvas.height = video.videoHeight;
            function draw() {
                if (video.paused || video.ended) return;
                normal_ctx.drawImage(video, 0, 0, normal_canvas.width, normal_canvas.height);
                requestAnimationFrame(draw);
            }
            draw();
        });
    })
    .catch(err => {
        console.error('エラー:', err);
    });

async function run() {
    await init();
    window.addEventListener("load", () => {
        function draw() {
            const imageData = normal_ctx.getImageData(0, 0, normal_canvas.width, normal_canvas.height);
            const ret = emoji_filter(new Uint8Array(imageData.data.buffer),normal_canvas.width,normal_canvas.height,16,false);
            document.getElementById("asciiCanvasDiv").innerText = ret;
            requestAnimationFrame(draw);
        }
        draw();
    });
    emoji_chk.addEventListener("change",(e)=>{
        function draw() {
            const imageData = normal_ctx.getImageData(0, 0, normal_canvas.width, normal_canvas.height);
            const ret = emoji_filter(new Uint8Array(imageData.data.buffer),normal_canvas.width,normal_canvas.height,16,e.target.checked);
            document.getElementById("asciiCanvasDiv").innerText = ret;
            requestAnimationFrame(draw);
        }
        draw();
    })
}
run();
