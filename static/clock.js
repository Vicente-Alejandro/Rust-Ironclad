// clock.js
const ClockModule = (function() {
    let currentMode = 'quartz'; 
    let requestRef;
    
    // 1. CATÁLOGO DINÁMICO DE RELOJES
    const WATCH_CATALOG = {
        quartz: {
            desc: 'Quartz Module 1Hz<br>Cushion Case',
            isDigital: false,
            bph: null, // El cuarzo salta a 1Hz
            template: `
                <div class="inner-bezel"></div>
                <div id="hour-markers"></div>
                <div class="watch-brand">Ironclad</div>
                <div class="watch-model">QUARTZ</div>
                <div class="watch-specs">WR 100M<br>FULL IRON</div>
                <div class="date-window"><span class="date-number" id="date-display">--</span></div>
                <div class="hands-container">
                    <div class="hand-hour" id="hand-hour"></div>
                    <div class="hand-minute" id="hand-minute"></div>
                    <div class="hand-second" id="hand-second"></div>
                    <div class="center-pin"></div>
                </div>
                <div class="glass-reflection"></div>
            `
        },
        automatic: {
            desc: 'Mechanical Module 18800 BPH<br>Open Heart Case',
            isDigital: false,
            bph: 18800, // El automático típico late a 18800 BPH (5.22Hz) saltos por segundo (aprox 5.22 saltos por segundo)
            template: `
                <div class="inner-bezel"></div>
                <div id="hour-markers"></div>
                <div class="watch-brand">Ironclad</div>
                <div class="watch-model">AUTOMATIC</div>
                <div class="watch-specs">24 JEWELS<br>18800 BPH</div>
                <div class="open-heart"><div class="gear"></div></div>
                <div class="date-window"><span class="date-number" id="date-display">--</span></div>
                <div class="hands-container">
                    <div class="hand-hour" id="hand-hour"></div>
                    <div class="hand-minute" id="hand-minute"></div>
                    <div class="hand-second" id="hand-second"></div>
                    <div class="center-pin"></div>
                </div>
                <div class="glass-reflection"></div>
            `
        },
        digital: {
            desc: 'Illuminator Module<br>Resin Square Case',
            isDigital: true,
            template: `
                <div class="w800-bezel">
                    <div class="w800-brand">IRONCLAD</div>
                    <div class="w800-illuminator">ILLUMINATOR</div>
                    <div class="w800-wr">WATER 100M RESIST</div>
                    <div class="w800-lcd">
                        <div class="lcd-header">
                            <span id="lcd-year">2026</span>
                            <span id="lcd-date">10-25</span>
                            <span id="lcd-day">MON</span>
                        </div>
                        <div class="lcd-time-row">
                            <span id="lcd-hour">10</span><span class="lcd-colon">:</span><span id="lcd-minute">58</span>
                            <span id="lcd-second">34</span>
                        </div>
                    </div>
                </div>
                <div class="glass-reflection"></div>
            `
        }
    };

    function init() {
        const select = document.getElementById('watch-style-select');
        select.addEventListener('change', (e) => renderWatch(e.target.value));
        renderWatch(select.value);
        requestRef = requestAnimationFrame(updateLoop);
    }

    function renderWatch(mode) {
        currentMode = mode;
        const watch = WATCH_CATALOG[mode];
        
        document.getElementById('watch-case').className = `watch-case ${mode}`;
        document.getElementById('watch-description').innerHTML = watch.desc;
        document.getElementById('watch-face').innerHTML = watch.template;

        if (!watch.isDigital) {
            const hourMarkers = document.getElementById('hour-markers');
            for (let i = 0; i < 12; i++) {
                const marker = document.createElement('div');
                marker.className = i % 3 === 0 ? 'hour-marker major' : 'hour-marker';
                marker.style.transform = `translateX(-50%) rotate(${i * 30}deg)`;
                hourMarkers.appendChild(marker);
            }
        }
    }
    
    function updateLoop() {
        const now = new Date();
        const watch = WATCH_CATALOG[currentMode];

        if (watch.isDigital) {
            const days = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
            document.getElementById('lcd-year').textContent = now.getFullYear();
            document.getElementById('lcd-date').textContent = `${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
            document.getElementById('lcd-day').textContent = days[now.getDay()];
            document.getElementById('lcd-hour').textContent = String(now.getHours()).padStart(2, '0');
            document.getElementById('lcd-minute').textContent = String(now.getMinutes()).padStart(2, '0');
            document.getElementById('lcd-second').textContent = String(now.getSeconds()).padStart(2, '0');
        } else {
            const hours = now.getHours();
            const minutes = now.getMinutes();
            const seconds = now.getSeconds();
            
            const hourAngle = (hours % 12) * 30 + minutes * 0.5;
            const minuteAngle = minutes * 6 + seconds * 0.1;
            
            let secondAngle = 0;
            
            if (watch.bph) {
                // FÍSICA AUTOMÁTICA DINÁMICA
                const beatsPerSecond = watch.bph / 3600;
                const msPerBeat = 1000 / beatsPerSecond;
                const degreesPerBeat = 6 / beatsPerSecond;
                
                const totalMs = now.getTime();
                const beats = Math.floor(totalMs / msPerBeat);
                secondAngle = (beats * degreesPerBeat) % 360;
            } else {
                // FÍSICA CUARZO
                secondAngle = seconds * 6;
            }
            
            document.getElementById('hand-hour').style.transform = `translateX(-50%) rotate(${hourAngle}deg)`;
            document.getElementById('hand-minute').style.transform = `translateX(-50%) rotate(${minuteAngle}deg)`;
            document.getElementById('hand-second').style.transform = `translateX(-50%) rotate(${secondAngle}deg)`;
            document.getElementById('date-display').textContent = now.getDate();
        }
        
        document.getElementById('digital-time').textContent = now.toLocaleTimeString('en-US', { 
            hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit'
        });
        
        requestRef = requestAnimationFrame(updateLoop);
    }
    
    return { init };
})();