// clock.js - Watch display module
const ClockModule = (function() {
    let currentMode = 'quartz'; 
    let requestRef;
    let isMaximized = false;
    
    // --- CALCULATOR STATE ---
    let calcState = { display: '0', operator: null, firstOperand: null, waitingForNewValue: false, isCalcMode: false };
    function loadCalcState() { const saved = localStorage.getItem('ironclad_calc_state'); if (saved) { try { calcState = JSON.parse(saved); } catch(e) {} } }
    function saveCalcState() { localStorage.setItem('ironclad_calc_state', JSON.stringify(calcState)); }

    // --- ESTADO DEL CASIOTRON TRN-50 ---
    let casiotronState = {
        mode: 0, // 0: TIME, 1: WT, 2: STW, 3: TMR, 4: ALM
        stw: { running: false, start: 0, elapsed: 0 },
        tmr: { running: false, end: 0, remaining: 10 * 60 * 1000, default: 10 * 60 * 1000 }, // 10 mins por defecto
        light: false
    };
    
    // --- SOLAR LIGHT ENGINE (SUN PATH SIMULATOR) ---
    let userLocation = { lat: -29.9045, lon: -71.2489 }; // Default: La Serena
    let lastSunUpdate = 0; 
    
    function requestLocation() {
        if ("geolocation" in navigator) {
            navigator.geolocation.getCurrentPosition(
                (position) => {
                    userLocation.lat = position.coords.latitude;
                    userLocation.lon = position.coords.longitude;
                    console.log("Ironclad Sun Engine: Using real coordinates.", userLocation);
                    // Force immediate update when getting location
                    updateSunlightReflection(new Date(), true); 
                },
                (error) => { console.log("Ironclad Sun Engine: Using default coordinates."); }
            );
        }
    }

    function calculateSunPosition(date, lat, lon) {
        const PI = Math.PI;
        const rad = PI / 180;
        const start = new Date(date.getFullYear(), 0, 0);
        const diff = date - start;
        const oneDay = 1000 * 60 * 60 * 24;
        const dayOfYear = Math.floor(diff / oneDay);
        const declination = -23.45 * Math.cos(rad * (360 / 365) * (dayOfYear + 10));
        const tzOffset = date.getTimezoneOffset() / 60; 
        const localTime = date.getHours() + (date.getMinutes() / 60) + (date.getSeconds() / 3600);
        const solarTime = localTime + (lon / 15) + tzOffset;
        const hourAngle = 15 * (solarTime - 12);
        
        const sinElevation = Math.sin(lat * rad) * Math.sin(declination * rad) + Math.cos(lat * rad) * Math.cos(declination * rad) * Math.cos(hourAngle * rad);
        const elevation = Math.asin(sinElevation) / rad;
        
        let azimut = Math.acos( (Math.sin(declination * rad) - Math.sin(lat * rad) * Math.sin(elevation * rad)) / (Math.cos(lat * rad) * Math.cos(elevation * rad)) ) / rad;
        if (hourAngle > 0) { azimut = 360 - azimut; }
        
        return { elevation, azimut };
    }

    function updateSunlightReflection(now, force = false) {
        // Recalculate every 5 seconds to save resources, unless forced
        if (!force && now.getTime() - lastSunUpdate < 5000) return; 
        lastSunUpdate = now.getTime();

        const sun = calculateSunPosition(now, userLocation.lat, userLocation.lon);
        
        let intensity1 = 0;
        let intensity2 = 0;

        if (sun.elevation > 0) {
            // The sun is above the horizon. We maximize at 0.4 if it exceeds 20 degrees elevation.
            const normalizationFactor = Math.min(sun.elevation / 20, 1);
            intensity1 = 0.4 * normalizationFactor;
            intensity2 = 0.1 * normalizationFactor;
        } 

        let lightAngleCSS = sun.azimut - 180;
        
        const radAzimut = (sun.azimut - 90) * (Math.PI / 180);
        const dist = 50 - Math.min(sun.elevation, 50); 
        const xPercent = 50 + (Math.cos(radAzimut) * dist);
        const yPercent = 50 + (Math.sin(radAzimut) * dist);

        document.documentElement.style.setProperty('--light-angle', `${lightAngleCSS}deg`);
        document.documentElement.style.setProperty('--light-intensity-1', intensity1);
        document.documentElement.style.setProperty('--light-intensity-2', intensity2);
        document.documentElement.style.setProperty('--light-x', `${xPercent}%`);
        document.documentElement.style.setProperty('--light-y', `${yPercent}%`);
    }

    // --- DYNAMIC CATALOG ---
    const WATCH_CATALOG = {
        quartz: {
            desc: 'Quartz Module 1Hz<br>Cushion Case',
            isDigital: false, bph: null, hideMarkers: [3],
            template: `<div class="watch-crown"></div><div class="watch-face"><div class="inner-bezel"></div><div id="hour-markers"></div><div class="watch-brand">Ironclad</div><div class="watch-model">QUARTZ</div><div class="watch-specs">WR 100M<br>FULL IRON</div><div class="date-window"><span class="date-number" id="date-display">--</span></div><div class="hands-container"><div class="hand-hour" id="hand-hour"></div><div class="hand-minute" id="hand-minute"></div><div class="hand-second" id="hand-second"></div><div class="center-pin"></div></div><div class="glass-reflection"></div></div>`
        },
        casiotron: {
            desc: 'TRN-50 50th Anniversary<br>Tough Solar & Multi-Mode',
            isDigital: true,
            hideMarkers: [],
            template: `
                <div class="casio-btn btn-a" data-btn="A"></div> <div class="casio-btn btn-b" data-btn="B"></div> <div class="casio-btn btn-c" data-btn="C"></div> <div class="casio-btn btn-d" data-btn="D"></div> <div class="casiotron-bezel"></div>
                <div class="casiotron-dial-container">
                    <div class="casiotron-dial-pattern"></div>
                        <div class="casiotron-gold-ring">

                            <div class="casiotron-logo">CASIO</div>

                            <div class="casiotron-lcd-frame">
                                <div class="casiotron-lcd" id="casiotron-lcd">
                                    <div class="casio-top-row">
                                        <span class="casio-ps">PS</span>
                                        <span class="casio-date-matrix" id="casio-header">SU 6.30</span>
                                    </div>
                                    <div class="casio-indicators-row">
                                        <span>LT</span><span id="casio-ind-1" class="active">RCVD</span><span>SNZ</span><span>ALM</span><span>SIG</span><span>MUTE</span><span class="casio-red">LOW</span>
                                    </div>
                                    <div class="casio-bottom-row">
                                        <span class="casio-pm" id="casio-pm">P</span>
                                        <span class="casio-main" id="casio-main">10:58</span>
                                        <span class="casio-sec" id="casio-sec">50</span>
                                    </div>
                                </div>
                            </div>
                            <div class="casiotron-model">
                                CASIOTRON
                                <div class="casiotron-sublogo">TRN-50 ANNIVERSARY</div>
                                <br>
                            </div>
                        </div>
                    </div>
                    <span class="casiotron-japan">JAPAN</span>
                </div>
                <div class="glass-reflection casiotron-glass"></div>
            `,
            onMount: function() {
                // Mapeo lógico de botones físicos
                document.querySelectorAll('.casio-btn').forEach(btn => {
                    btn.addEventListener('pointerdown', (e) => {
                        e.preventDefault(); // Evita comportamientos fantasma en móviles
                        e.stopPropagation();
                        const id = e.target.dataset.btn;
                        
                        if (id === 'C') {
                            // BOTÓN C (Bottom Left): Cambiar Modo
                            casiotronState.mode = (casiotronState.mode + 1) % 5;
                        } else if (id === 'D') {
                            // BOTÓN D (Bottom Right): Start/Stop
                            if (casiotronState.mode === 2) { // STW
                                if (casiotronState.stw.running) {
                                    casiotronState.stw.running = false;
                                    casiotronState.stw.elapsed += Date.now() - casiotronState.stw.start;
                                } else {
                                    casiotronState.stw.running = true;
                                    casiotronState.stw.start = Date.now();
                                }
                            } else if (casiotronState.mode === 3) { // TMR
                                if (casiotronState.tmr.running) {
                                    casiotronState.tmr.running = false;
                                    casiotronState.tmr.remaining = casiotronState.tmr.end - Date.now();
                                } else {
                                    casiotronState.tmr.running = true;
                                    casiotronState.tmr.end = Date.now() + casiotronState.tmr.remaining;
                                }
                            }
                        } else if (id === 'A') {
                            // BOTÓN A (Top Left): Reset
                            if (casiotronState.mode === 2) {
                                casiotronState.stw = { running: false, start: 0, elapsed: 0 };
                            } else if (casiotronState.mode === 3) {
                                casiotronState.tmr = { running: false, end: 0, remaining: casiotronState.tmr.default, default: casiotronState.tmr.default };
                            }
                        } else if (id === 'B') {
                            // BOTÓN B (Top Right): Super Illuminator LED
                            casiotronState.light = true;
                            setTimeout(() => casiotronState.light = false, 2000); // Se apaga a los 2 seg
                        }
                    });
                });
            }
        },
        grandseiko: {
            desc: 'Hi-Beat 36000 BPH<br>Zaratsu Polish & Dauphine Hands',
            isDigital: false,
            bph: 36000, // 10 saltos exactos por segundo (Hi-Beat)
            hideMarkers: [3],
            template: `
                <div class="watch-crown gs-crown"></div>
                <div class="gs-bezel"></div>
                
                <div class="watch-face gs-face">
                    <div class="gs-texture"></div>
                    
                    <div id="hour-markers"></div>
                    
                    <div class="watch-brand gs-brand">GS<br><span class="gs-sub">grand seiko</span></div>
                    <div class="watch-specs gs-specs">HI-BEAT 36000 <div class="gs-specs-red">GMT</div></div>
                    
                    <div class="date-window gs-date"><span class="date-number" id="date-display">--</span></div>
                    
                    <div class="hands-container">
                        <div class="hand-hour gs-hour" id="hand-hour"></div>
                        <div class="hand-minute gs-minute" id="hand-minute"></div>
                        <div class="hand-second gs-second" id="hand-second"></div>
                        <div class="center-pin gs-pin"></div>
                    </div>
                </div>
                <div class="glass-reflection"></div>
            `
        },
        submariner: {
            desc: 'Swiss GMT Chronometer 28800 BPH<br>Bicolor Ceramic Bezel',
            isDigital: false,
            bph: 28800,
            hideMarkers: [3],
            template: `
                <div class="watch-crown submariner-crown"></div>
                <div class="submariner-bezel">
                    <div id="gmt-bezel-numbers" class="gmt-numbers"></div>
                </div>
                
                <div class="watch-face submariner-face">
                    <div id="hour-markers"></div>
                    <div class="watch-brand submariner-brand">ROLEX</div>
                    <div class="watch-model submariner-model">OYSTER PERPETUAL IRON</div>
                    <div class="watch-specs submariner-specs"><div class="r">SUBMARINER</div>1000ft = 300<span style="font-style:italic">m</span><br>SUPERLATIVE CHRONOMETER<br>OFFICIALLY CERTIFIED</div>
                    <div class="date-window submariner-date"><span class="date-number" id="date-display">--</span></div>
                    
                    <div class="hands-container">
                        <div class="hand-hour submariner-hour" id="hand-hour"><div class="mercedes-circle"></div></div>
                        <div class="hand-minute submariner-minute" id="hand-minute"></div>
                        <div class="hand-second submariner-second" id="hand-second"></div>
                        <div class="center-pin submariner-pin"></div>
                    </div>
                </div>
                <div class="glass-reflection"></div>
                <div class="submariner-cyclops"></div>
            `,
            // Ejecutamos código para inyectar los números matemáticamente al cargar el reloj
            onMount: function () {
                const gmtContainer = document.getElementById('gmt-bezel-numbers');
                if (!gmtContainer) return;

                // Nivel Experto: Array de tu bisel Dive/GMT personalizado
                const gmtMarks = [
                    '▼', '▮', '1', '▮', '2', '▮', '3', '',
                    '4', '', '5', '', '6', '', '7', '',
                    '8', '', '9', '', '10', '', '11', ''
                ];

                gmtContainer.innerHTML = '';

                gmtMarks.forEach((mark, index) => {
                    const numDiv = document.createElement('div');
                    numDiv.className = 'gmt-num';
                    numDiv.style.transform = `rotate(${index * 15}deg)`;

                    const innerSpan = document.createElement('span');
                    innerSpan.textContent = mark;

                    if (mark === '▮') {
                        innerSpan.className = 'gmt-dot';
                    } else if (mark === '■') {
                        innerSpan.className = 'gmt-dot-thick';
                    }

                    numDiv.appendChild(innerSpan);
                    gmtContainer.appendChild(numDiv);
                });

                // --- LÓGICA DE BISEL ROTATORIO (DIVE BEZEL) ---
                const bezelElement = document.querySelector('.submariner-bezel');
                if (!bezelElement) return;

                // Recuperar la rotación guardada (o 0 por defecto)
                let currentRotation = parseFloat(localStorage.getItem('ironclad_submariner_bezel')) || 0;
                bezelElement.style.transform = `rotate(${currentRotation}deg)`;

                // Función para girar el bisel (Unidireccional: 120 clicks = 3 grados por click)
                const rotateBezel = (degrees) => {
                    currentRotation += degrees;
                    // Mantenemos el número entre -360 y 0 para que no crezca infinitamente
                    if (currentRotation <= -360) currentRotation = 0;

                    bezelElement.style.transform = `rotate(${currentRotation}deg)`;
                    localStorage.setItem('ironclad_submariner_bezel', currentRotation);
                };

                // Asignar evento de clic al bisel
                bezelElement.addEventListener('mousedown', (e) => {
                    e.stopPropagation(); // Evitar cerrar el modal maximizado

                    // Obtener las dimensiones del bisel para saber de qué lado hicimos clic
                    const rect = bezelElement.getBoundingClientRect();
                    const centerX = rect.left + rect.width / 2;

                    // Si hacemos clic, simulamos el giro manual. 
                    // Unidireccional hacia la izquierda (-3 grados = 1 click de Rolex)
                    // Si quisieras que gire rápido si dejas el mouse apretado, requeriría un setInterval, 
                    // pero para mayor control, un clic = un salto de 3 a 15 grados es más realista en UI.

                    rotateBezel(-15); // Rotamos 5 clics de golpe (15 grados) para que sea cómodo de usar
                });

                // Soporte táctil para móviles
                bezelElement.addEventListener('touchstart', (e) => {
                    e.preventDefault(); // Evitar scroll de pantalla
                    e.stopPropagation();
                    rotateBezel(-15);
                }, { passive: false });
            }
        },
        automatic: {
            desc: 'Mechanical Module 18800 BPH<br>Open Heart Case',
            isDigital: false, bph: 18800, hideMarkers: [3, 6],
            template: `<div class="watch-crown"></div><div class="watch-face"><div class="inner-bezel"></div><div id="hour-markers"></div><div class="watch-brand">Ironclad</div><div class="watch-model">AUTOMATIC</div><div class="watch-specs">24 JEWELS<br>SAPPHIRE<br>jap mov</div></div></div><div class="date-window"><span class="date-number" id="date-display">--</span></div><div class="hands-container"><div class="hand-hour" id="hand-hour"></div><div class="hand-minute" id="hand-minute"></div><div class="hand-second" id="hand-second"></div><div class="center-pin"></div></div><div class="glass-reflection"></div></div>`
        },
        vostok: {
            desc: 'Russian Diver 19800 BPH<br>Domed Acrylic Crystal',
            isDigital: false, bph: 19800, hideMarkers: [3],
            template: `<div class="watch-crown vostok-crown"></div><div class="watch-face vostok-face"><div id="hour-markers"></div><div class="watch-brand vostok-brand">IRONCLAD</div><div class="watch-model vostok-model">AMPHIBIA</div><div class="watch-specs vostok-specs">200M<br>31 JEWELS</div><div class="date-window vostok-date"><span class="date-number" id="date-display">--</span></div><div class="hands-container"><div class="hand-hour vostok-hour" id="hand-hour"></div><div class="hand-minute vostok-minute" id="hand-minute"></div><div class="hand-second vostok-second" id="hand-second"></div><div class="center-pin vostok-pin"></div></div></div><div class="glass-reflection domed-acrylic"></div>`
        },
        sbsa255: {
            desc: 'Seiko 5 Sports JDM 21600 BPH<br>37.4mm Field/Diver Case',
            isDigital: false, bph: 21600, hideMarkers: [3],
            template: `<div class="watch-crown sbsa-crown"></div><div class="sbsa-bezel"></div><div class="watch-face sbsa-face"><div id="hour-markers"></div><div class="watch-brand sbsa-brand"><span class="sbsa-5">SEIKO SPORT</span></div><div class="watch-model sbsa-model">FULL IRON<br>AUTOMATIC<br><div></div></div><div class="watch-specs sbsa-specs"><div class="gs-specs-red">100M WR</div>MADE IN JAPAN</div><div class="date-window sbsa-date"><span class="date-number" id="date-display">--</span></div><div class="hands-container"><div class="hand-hour sbsa-hour" id="hand-hour"></div><div class="hand-minute sbsa-minute" id="hand-minute"></div><div class="hand-second sbsa-second" id="hand-second"></div><div class="center-pin sbsa-pin"></div></div></div><div class="glass-reflection"></div>`
        },
        seaclad: {
            desc: 'Co-Axial Master 25200 BPH<br>Wave Dial & Helium Valve',
            isDigital: false, bph: 25200, hideMarkers: [6], 
            template: `<div class="watch-crown seaclad-crown"></div></div><div class="seaclad-bezel"></div><div class="watch-face seaclad-face"><div class="seaclad-waves"></div><div id="hour-markers"></div><div class="watch-brand seaclad-brand">IRONCLAD</div><div class="watch-model seaclad-model">SEACLAD<br>PROFESSIONAL</div><div class="watch-specs seaclad-specs">CO-AXIAL MASTER<br>300m / 1000ft</div><div class="date-window seaclad-date"><span class="date-number" id="date-display">--</span></div><div class="hands-container"><div class="hand-hour seaclad-hour" id="hand-hour"></div><div class="hand-minute seaclad-minute" id="hand-minute"></div><div class="hand-second seaclad-second" id="hand-second"></div><div class="center-pin seaclad-pin"></div></div></div><div class="glass-reflection"></div>`
        },
        accutron: {
            desc: 'Tuning Fork Module 360Hz<br>Spaceview Circuitry',
            isDigital: false, bph: 'smooth', hideMarkers: [],
            template: `<div class="watch-face accutron-face"><div class="accutron-pcb"><div class="pcb-trace trace-1"></div><div class="pcb-trace trace-2"></div><div class="accutron-coil coil-left"></div><div class="accutron-coil coil-right"></div><div class="accutron-component comp-1"></div><div class="accutron-component comp-2"></div><div class="tuning-fork"><div class="tf-tine tine-left"></div><div class="tf-tine tine-right"></div></div></div><div class="chapter-ring"><div class="watch-brand accutron-brand">IRONCLAD</div><div class="watch-model accutron-model">SPACEVIEW</div></div><div id="hour-markers"></div><div class="hands-container"><div class="hand-hour accutron-hour" id="hand-hour"></div><div class="hand-minute accutron-minute" id="hand-minute"></div><div class="hand-second accutron-second" id="hand-second"></div><div class="center-pin accutron-pin"></div></div><div class="glass-reflection accutron-glass"></div></div>`
        },
        digital: {
            desc: 'Illuminator Module<br>Resin Square Case',
            isDigital: true, hideMarkers: [],
            template: `<div class="w800-bezel"><div class="w800-brand">IRONCLAD</div><div class="w800-illuminator">ILLUMINATOR</div><div class="w800-wr">WATER 100M RESIST</div><div class="w800-lcd"><div class="lcd-header"><span id="lcd-year">2026</span><span id="lcd-date">10-25</span><span id="lcd-day">MON</span></div><div class="lcd-time-row"><span id="lcd-hour">10</span><span class="lcd-colon">:</span><span id="lcd-minute">58</span><span id="lcd-second">34</span></div></div><div class="glass-reflection digital-glass"></div></div>`
        },
        databank: {
            desc: 'Calculator Watch Module<br>Resin Case with Keypad',
            isDigital: true, hideMarkers: [],
            template: `<div class="dbc-bezel"><div class="dbc-screen-area"><div class="dbc-brand">IRONCLAD <span class="dbc-sub">DATABANK</span></div><div class="dbc-lcd" id="dbc-lcd"><div id="dbc-time-mode"><div class="dbc-header"><span id="dbc-year">2026</span><span id="dbc-date">10-25</span><span id="dbc-day">MON</span></div><div class="dbc-time-row"><span id="dbc-hour">10</span><span class="dbc-colon">:</span><span id="dbc-minute">58</span><span id="dbc-second">34</span></div></div><div id="dbc-calc-mode" style="display: none;"><div class="dbc-calc-indicator">CALC</div><div class="dbc-calc-display" id="dbc-calc-display">0</div></div></div></div><div class="dbc-keypad"><button class="dbc-key" data-action="mode">MODE</button><button class="dbc-key" data-num="7">7</button><button class="dbc-key" data-num="8">8</button><button class="dbc-key" data-num="9">9</button><button class="dbc-key dbc-op" data-op="/">÷</button><button class="dbc-key" data-action="clear">C</button><button class="dbc-key" data-num="4">4</button><button class="dbc-key" data-num="5">5</button><button class="dbc-key" data-num="6">6</button><button class="dbc-key dbc-op" data-op="*">×</button><button class="dbc-key" data-num="0">0</button><button class="dbc-key" data-num="1">1</button><button class="dbc-key" data-num="2">2</button><button class="dbc-key" data-num="3">3</button><button class="dbc-key dbc-op" data-op="-">-</button><button class="dbc-key" data-num=".">.</button><button class="dbc-key dbc-eq" data-action="calculate">=</button><button class="dbc-key dbc-op" data-op="+">+</button></div><div class="glass-reflection dbc-glass"></div></div>`,
            onMount: function() {
                loadCalcState();
                const timeModeEl = document.getElementById('dbc-time-mode');
                const calcModeEl = document.getElementById('dbc-calc-mode');
                const calcDisplayEl = document.getElementById('dbc-calc-display');
                
                timeModeEl.style.display = calcState.isCalcMode ? 'none' : 'block';
                calcModeEl.style.display = calcState.isCalcMode ? 'block' : 'none';
                calcDisplayEl.textContent = calcState.display;
                
                const calculate = (n1, operator, n2) => {
                    const firstNum = parseFloat(n1);
                    const secondNum = parseFloat(n2);
                    if (operator === '+') return firstNum + secondNum;
                    if (operator === '-') return firstNum - secondNum;
                    if (operator === '*') return firstNum * secondNum;
                    if (operator === '/') return secondNum === 0 ? 'ERR' : firstNum / secondNum;
                    return secondNum;
                };

                document.querySelectorAll('.dbc-key').forEach(button => {
                    button.addEventListener('click', (e) => {
                        e.stopPropagation(); 
                        const val = e.target.dataset.num;
                        const action = e.target.dataset.action;
                        const op = e.target.dataset.op;

                        if (action === 'mode') {
                            calcState.isCalcMode = !calcState.isCalcMode;
                            timeModeEl.style.display = calcState.isCalcMode ? 'none' : 'block';
                            calcModeEl.style.display = calcState.isCalcMode ? 'block' : 'none';
                            saveCalcState();
                            return;
                        }

                        if (!calcState.isCalcMode) return;

                        if (val !== undefined) {
                            if (calcState.waitingForNewValue) {
                                calcState.display = val;
                                calcState.waitingForNewValue = false;
                            } else {
                                calcState.display = calcState.display === '0' ? val : calcState.display + val;
                            }
                        }

                        if (op !== undefined) {
                            const inputValue = parseFloat(calcState.display);
                            if (calcState.firstOperand === null && !isNaN(inputValue)) {
                                calcState.firstOperand = inputValue;
                            } else if (calcState.operator) {
                                const result = calculate(calcState.firstOperand, calcState.operator, inputValue);
                                calcState.display = String(result).substring(0, 8); 
                                calcState.firstOperand = result;
                            }
                            calcState.operator = op;
                            calcState.waitingForNewValue = true;
                        }

                        if (action === 'calculate') {
                            if (calcState.operator && !calcState.waitingForNewValue) {
                                const result = calculate(calcState.firstOperand, calcState.operator, parseFloat(calcState.display));
                                calcState.display = String(result).substring(0, 8);
                                calcState.firstOperand = null;
                                calcState.operator = null;
                                calcState.waitingForNewValue = true;
                            }
                        }

                        if (action === 'clear') {
                            calcState.display = '0';
                            calcState.firstOperand = null;
                            calcState.operator = null;
                            calcState.waitingForNewValue = false;
                        }

                        calcDisplayEl.textContent = calcState.display;
                        saveCalcState();
                    });
                });
            }
        }
    };

    function init() {
        const select = document.getElementById('watch-style-select');
        const savedMode = localStorage.getItem('ironclad_watch_mode');
        
        if (savedMode && WATCH_CATALOG[savedMode]) {
            select.value = savedMode;
        }

        // Request location to simulate real sun
        requestLocation();

        select.addEventListener('change', (e) => {
            const newMode = e.target.value;
            localStorage.setItem('ironclad_watch_mode', newMode);
            renderWatch(newMode);
        });

        setupMaximizeFeature();
        renderWatch(select.value);
        requestRef = requestAnimationFrame(updateLoop);
    }

    function setupMaximizeFeature() {
        const maximizeBtn = document.getElementById('maximize-watch-btn');
        const closeModalBtn = document.getElementById('close-watch-modal');
        const modalOverlay = document.getElementById('watch-modal');
        const originalContainer = document.getElementById('original-watch-container');
        const maximizedContainer = document.getElementById('maximized-watch-container');
        const watchCase = document.getElementById('watch-case');

        function openModal() {
            isMaximized = true;
            maximizedContainer.appendChild(watchCase);
            modalOverlay.classList.add('active');
        }

        function closeModal() {
            isMaximized = false;
            modalOverlay.classList.remove('active');
            setTimeout(() => { originalContainer.appendChild(watchCase); }, 300);
        }

        maximizeBtn.addEventListener('click', openModal);
        closeModalBtn.addEventListener('click', closeModal);
        modalOverlay.addEventListener('click', (e) => { if (e.target === modalOverlay) closeModal(); });
        document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && isMaximized) closeModal(); });
    }

    function renderWatch(mode) {
        const watch = WATCH_CATALOG[mode];
        if (!watch) return; 

        currentMode = mode;
        document.getElementById('watch-case').className = `watch-case ${mode}`;
        document.getElementById('watch-description').innerHTML = watch.desc;
        document.getElementById('watch-case').innerHTML = watch.template;

        if (!watch.isDigital) {
            const hourMarkers = document.getElementById('hour-markers');
            if (hourMarkers) {
                const hidden = watch.hideMarkers || [];
                for (let i = 0; i < 12; i++) {
                    const marker = document.createElement('div');
                    marker.className = i % 3 === 0 ? 'hour-marker major' : 'hour-marker';
                    marker.style.transform = `translateX(-50%) rotate(${i * 30}deg)`;
                    if (hidden.includes(i)) marker.style.display = 'none';
                    hourMarkers.appendChild(marker);
                }
            }
        }
        if (typeof watch.onMount === 'function') watch.onMount();
        
        // Force a light update when changing watches
        updateSunlightReflection(new Date(), true);
    }
    
    function updateLoop() {
        const now = new Date();
        const watch = WATCH_CATALOG[currentMode];
        if (!watch) { requestRef = requestAnimationFrame(updateLoop); return; }

        // --- UPDATE SOLAR ENGINE ---
        updateSunlightReflection(now);

        if (currentMode === 'casiotron') {
            // --- CASIOTRON BRAIN ---
            const headerEl = document.getElementById('casio-header');
            const mainEl = document.getElementById('casio-main');
            const secEl = document.getElementById('casio-sec');
            const lcdEl = document.getElementById('casiotron-lcd');
            const ind1El = document.getElementById('casio-ind-1');
            if (!headerEl) return;

            // Illumination
            if (casiotronState.light) lcdEl.classList.add('illuminated');
            else lcdEl.classList.remove('illuminated');

            if (casiotronState.mode === 0) {
                // MODO: TIME
                const days = ["SU", "MO", "TU", "WE", "TH", "FR", "SA"];
                const isPM = now.getHours() >= 12;
                const displayHour = now.getHours() % 12 || 12; // Formato 12 hrs

                // Mostrar u ocultar la 'P'
                document.getElementById('casio-pm').style.visibility = isPM ? 'visible' : 'hidden';

                // Fecha formato 6.30 (Mes.Día)
                headerEl.textContent = `${days[now.getDay()]} ${now.getMonth() + 1}.${String(now.getDate()).padStart(2, '0')}`;

                mainEl.textContent = `${String(displayHour).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
                secEl.textContent = String(now.getSeconds()).padStart(2, '0');
                ind1El.textContent = "RCVD";
                ind1El.classList.add('active');
            } else if (casiotronState.mode === 1) {
                // MODE: WORLD TIME (Simulating Tokyo +9)
                headerEl.textContent = "WT TYO";
                const tokyoTime = new Date(now.getTime() + (12 * 60 * 60 * 1000));
                mainEl.textContent = `${String(tokyoTime.getHours()).padStart(2, '0')}:${String(tokyoTime.getMinutes()).padStart(2, '0')}`;
                secEl.textContent = String(tokyoTime.getSeconds()).padStart(2, '0');
                ind1El.textContent = "WT";
            } else if (casiotronState.mode === 2) {
                // MODE: STOPWATCH (Millisecond precision)
                headerEl.textContent = "STW";
                let totalMs = casiotronState.stw.elapsed;
                if (casiotronState.stw.running) totalMs += now.getTime() - casiotronState.stw.start;

                const ms = Math.floor((totalMs % 1000) / 10);
                const s = Math.floor((totalMs / 1000) % 60);
                const m = Math.floor((totalMs / 60000) % 60);
                mainEl.textContent = `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
                secEl.textContent = String(ms).padStart(2, '0');
                ind1El.textContent = casiotronState.stw.running ? "RUN" : "STP";
            } else if (casiotronState.mode === 3) {
                // MODE: TIMER
                headerEl.textContent = "TMR";
                let remain = casiotronState.tmr.remaining;
                if (casiotronState.tmr.running) {
                    remain = casiotronState.tmr.end - now.getTime();
                    if (remain <= 0) { remain = 0; casiotronState.tmr.running = false; }
                }
                const s = Math.floor((remain / 1000) % 60);
                const m = Math.floor((remain / 60000) % 60);
                mainEl.textContent = `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
                secEl.textContent = "00";
                ind1El.textContent = casiotronState.tmr.running ? "RUN" : "STP";
            } else if (casiotronState.mode === 4) {
                // MODE: ALARM
                headerEl.textContent = "ALM";
                mainEl.textContent = "12:00";
                secEl.textContent = "ON";
                ind1El.textContent = "ALM";
            }

        } else if (watch.isDigital) {
            // --- NORMAL DIGITAL LOGIC (W800 / DATABANK) ---
            const days = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
            const prefix = currentMode === 'databank' ? 'dbc' : 'lcd';
            const yearEl = document.getElementById(`${prefix}-year`);
            if (yearEl) yearEl.textContent = now.getFullYear();
            const dateEl = document.getElementById(`${prefix}-date`);
            if (dateEl) dateEl.textContent = `${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
            const dayEl = document.getElementById(`${prefix}-day`);
            if (dayEl) dayEl.textContent = days[now.getDay()];
            const hourEl = document.getElementById(`${prefix}-hour`);
            if (hourEl) hourEl.textContent = String(now.getHours()).padStart(2, '0');
            const minEl = document.getElementById(`${prefix}-minute`);
            if (minEl) minEl.textContent = String(now.getMinutes()).padStart(2, '0');
            const secEl = document.getElementById(`${prefix}-second`);
            if (secEl) secEl.textContent = String(now.getSeconds()).padStart(2, '0');
        } else {
            const hours = now.getHours();
            const minutes = now.getMinutes();
            const seconds = now.getSeconds();
            const milliseconds = now.getMilliseconds();
            
            const hourAngle = (hours % 12) * 30 + minutes * 0.5;
            const minuteAngle = minutes * 6 + seconds * 0.1;
            
            let secondAngle = 0;
            if (watch.bph === 'smooth') {
                secondAngle = (seconds * 6) + (milliseconds * 0.006);
            } else if (watch.bph) {
                const beatsPerSecond = watch.bph / 3600;
                const msPerBeat = 1000 / beatsPerSecond;
                const degreesPerBeat = 6 / beatsPerSecond;
                const totalMs = now.getTime();
                const beats = Math.floor(totalMs / msPerBeat);
                secondAngle = (beats * degreesPerBeat) % 360;
            } else {
                secondAngle = seconds * 6;
            }
            
            const handHour = document.getElementById('hand-hour');
            if(handHour) handHour.style.transform = `translateX(-50%) rotate(${hourAngle}deg)`;
            const handMin = document.getElementById('hand-minute');
            if(handMin) handMin.style.transform = `translateX(-50%) rotate(${minuteAngle}deg)`;
            const handSec = document.getElementById('hand-second');
            if(handSec) handSec.style.transform = `translateX(-50%) rotate(${secondAngle}deg)`;
            
            const dateDisplay = document.getElementById('date-display');
            if(dateDisplay) dateDisplay.textContent = now.getDate();
        }
        
        document.getElementById('digital-time').textContent = now.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
        requestRef = requestAnimationFrame(updateLoop);
    }
    
    return { init };
})();