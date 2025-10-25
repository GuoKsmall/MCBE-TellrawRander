// Show error message
function showError(message) {
    const errorDiv = document.getElementById('error-message');
    if (!errorDiv) {
        console.warn('showError: no #error-message element in DOM. Message:', message);
        return;
    }
    errorDiv.textContent = message;
    errorDiv.style.display = 'block';
    setTimeout(() => {
        errorDiv.style.display = 'none';
    }, 5000);
}

// Render function
async function renderText(formData) {
    try {
        const response = await fetch('api/render', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'image/png, application/json'
            },
            body: JSON.stringify({
                mode: formData.get('mode'),
                content: formData.get('content').trim()
            })
        });

        if (!response.ok) {
            const contentType = response.headers.get('content-type');
            if (contentType && contentType.includes('application/json')) {
                const errorData = await response.json();
                throw new Error(errorData.error || 'Render request failed');
            } else {
                const errorText = await response.text();
                throw new Error(errorText || 'Render request failed');
            }
        }

        const contentType = response.headers.get('content-type');
        if (!contentType || !contentType.includes('image/png')) {
            throw new Error('Server returned wrong content type');
        }

        const blob = await response.blob();
        if (blob.size === 0) {
            throw new Error('Generated image is empty');
        }
        // Return blob for persistence
        return blob;
    } catch (error) {
        console.error('Render error:', error);
        throw error;
    }
}

// Handle form submission
function handleFormSubmit(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    const content = formData.get('content').trim();
    
    if (!content) {
        showError('Please enter content to render');
        return;
    }
    
    const previewId = form.id === 'text-form' ? 'text-preview' : 'json-preview';
    const preview = document.getElementById(previewId);
    const submitButton = form.querySelector('button[type="submit"]');
    const originalButtonText = submitButton.textContent;

    // Disable submit button and show loading state
    submitButton.disabled = true;
    submitButton.textContent = 'Rendering...';
    preview.innerHTML = '<div class="loading">Rendering content, please wait...</div>';

    // Restore preview background
    preview.style.backgroundColor = '';
    preview.style.setProperty('--preview-bg-opacity', '0.25');

    renderText(formData)
        .then(async (blob) => {
            // Convert blob to dataURL for persistence; use objectURL for display
            const objectUrl = URL.createObjectURL(blob);
            const dataUrl = await blobToDataURL(blob);
            const img = new Image();
            const isText = previewId === 'text-preview';
            const slider = document.getElementById(isText ? 'text-opacity' : 'json-opacity');
            const applyOpacityToPreview = (val) => {
                const opacityVal = Number(val) / 100;
                img.style.opacity = String(opacityVal);
                preview.style.setProperty('--preview-bg-opacity', String(opacityVal));
            };
            img.onload = () => {
                // Don't revoke dataUrl as it's a string; revoke objectUrl
                URL.revokeObjectURL(objectUrl);
                submitButton.disabled = false;
                submitButton.textContent = originalButtonText;
                if (slider) applyOpacityToPreview(slider.value);
                
                // Apply current transform settings
                applyTransformToPreview(form.querySelector('.offset-range') || form.querySelector('.scale-range'));
            };
            img.onerror = () => {
                showError('Image loading failed, check content format');
                preview.innerHTML = '<div class="placeholder">Render failed</div>';
                submitButton.disabled = false;
                submitButton.textContent = originalButtonText;
            };
            if (slider) applyOpacityToPreview(slider.value);
            img.src = objectUrl;
            preview.innerHTML = '';
            preview.appendChild(img);

            // Save history record (mode, content, dataUrl, timestamp)
            addHistoryRecord({
                mode: formData.get('mode'),
                content,
                dataUrl,
                time: Date.now()
            });
        })
        .catch(error => {
            console.error('Render error:', error);
            showError(error.message || 'Render failed, check content format');
            preview.innerHTML = '<div class="placeholder">Render failed</div>';
            submitButton.disabled = false;
            submitButton.textContent = originalButtonText;
        });
}

// Utility: blob -> dataURL
function blobToDataURL(blob) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onloadend = () => resolve(reader.result);
        reader.onerror = reject;
        reader.readAsDataURL(blob);
    });
}

/* History implementation: save to localStorage (max 30 items), clickable to restore */
const HISTORY_KEY = 'mcbe_text_impact_history_v1';
let historyList = [];

function loadHistory() {
    try {
        const raw = localStorage.getItem(HISTORY_KEY);
        historyList = raw ? JSON.parse(raw) : [];
    } catch (e) {
        historyList = [];
    }
}

function saveHistory() {
    try {
        localStorage.setItem(HISTORY_KEY, JSON.stringify(historyList));
    } catch (e) {
        console.warn('Cannot save history to localStorage', e);
    }
}

function addHistoryRecord(record) {
    // Deduplicate: keep latest for same mode + content
    historyList = historyList.filter(r => !(r.mode === record.mode && r.content === record.content));
    historyList.unshift(record);
    if (historyList.length > 30) historyList.length = 30;
    saveHistory();
    renderHistoryPanel();
}

function renderHistoryPanel() {
    const listEl = document.getElementById('history-list');
    const emptyEl = document.getElementById('history-empty');
    if (!listEl || !emptyEl) return;
    listEl.innerHTML = '';
    if (!historyList.length) {
        emptyEl.style.display = '';
        return;
    }
    emptyEl.style.display = 'none';
    historyList.forEach((item, idx) => {
        const li = document.createElement('li');
        const label = item.mode === 'text' ? 'Text' : 'JSON';
        const short = item.content.length > 40 ? item.content.slice(0, 40) + '…' : item.content;
        li.textContent = `${label} — ${short}`;
        li.title = item.content;
        li.addEventListener('click', () => showHistoryItem(idx));
        listEl.appendChild(li);
    });
}

function showHistoryItem(idx) {
    const item = historyList[idx];
    if (!item) return;
    // Highlight
    const listEl = document.getElementById('history-list');
    Array.from(listEl.children).forEach((li, i) => li.classList.toggle('active', i === idx));
    // Restore image and input
    const previewId = item.mode === 'text' ? 'text-preview' : 'json-preview';
    const inputId = item.mode === 'text' ? 'text-input' : 'json-input';
    const preview = document.getElementById(previewId);
    const input = document.getElementById(inputId);
    if (input) input.value = item.content;
    if (preview) {
        preview.innerHTML = '';
        const img = new Image();
        img.src = item.dataUrl; // data URL is permanently available
        img.style.maxWidth = '100%';
        img.onload = () => {};
        preview.appendChild(img);
    }
    
    // Reset transform controls to default
    const isText = item.mode === 'text';
    const offsetX = document.getElementById(isText ? 'text-offset-x' : 'json-offset-x');
    const offsetY = document.getElementById(isText ? 'text-offset-y' : 'json-offset-y');
    const scale = document.getElementById(isText ? 'text-scale' : 'json-scale');
    
    if (offsetX) {
        offsetX.value = 0;
        offsetX.dispatchEvent(new Event('input'));
    }
    if (offsetY) {
        offsetY.value = 0;
        offsetY.dispatchEvent(new Event('input'));
    }
    if (scale) {
        scale.value = 100;
        scale.dispatchEvent(new Event('input'));
    }
}

// Collapse state (save to localStorage)
const HISTORY_UI_KEY = 'mcbe_text_impact_history_ui_v1';
function loadHistoryUI() {
    try {
        const raw = localStorage.getItem(HISTORY_UI_KEY);
        return raw ? JSON.parse(raw) : { collapsed: false };
    } catch (e) { return { collapsed: false }; }
}

function saveHistoryUI(state) {
    try { localStorage.setItem(HISTORY_UI_KEY, JSON.stringify(state)); } catch (e) {}
}

function setupHistoryToggle() {
    const panel = document.getElementById('history-panel');
    const toggle = document.getElementById('history-toggle');
    if (!panel || !toggle) return;
    const uiState = loadHistoryUI();
    if (uiState.collapsed) {
        panel.classList.add('collapsed');
        toggle.setAttribute('aria-expanded', 'false');
        toggle.textContent = '▸';
    } else {
        panel.classList.remove('collapsed');
        toggle.setAttribute('aria-expanded', 'true');
        toggle.textContent = '▾';
    }
    toggle.addEventListener('click', () => {
        const collapsed = panel.classList.toggle('collapsed');
        toggle.setAttribute('aria-expanded', collapsed ? 'false' : 'true');
        toggle.textContent = collapsed ? '▸' : '▾';
        saveHistoryUI({ collapsed });
    });
}

// Handle form reset
function handleFormReset(event) {
    const form = event.target;
    const previewId = form.id === 'text-form' ? 'text-preview' : 'json-preview';
    const preview = document.getElementById(previewId);
    preview.innerHTML = '<div class="placeholder">(Render result)</div>';
    // Restore background and checkerboard transparency
    preview.style.backgroundColor = '';
    preview.style.setProperty('--preview-bg-opacity', '0.25');
    
    // Reset transform controls
    const isText = form.id === 'text-form';
    const offsetX = document.getElementById(isText ? 'text-offset-x' : 'json-offset-x');
    const offsetY = document.getElementById(isText ? 'text-offset-y' : 'json-offset-y');
    const scale = document.getElementById(isText ? 'text-scale' : 'json-scale');
    
    if (offsetX) {
        offsetX.value = 0;
        offsetX.dispatchEvent(new Event('input'));
    }
    if (offsetY) {
        offsetY.value = 0;
        offsetY.dispatchEvent(new Event('input'));
    }
    if (scale) {
        scale.value = 100;
        scale.dispatchEvent(new Event('input'));
    }
}

// Initialize: bind events and setup slider behavior
function initUI() {
    console.log('initUI: initializing UI bindings');
    // Bind form events
    document.querySelectorAll('.render-form').forEach(form => {
        console.log('initUI: binding form', form.id);
        form.addEventListener('submit', event => {
            handleFormSubmit(event);
        });
        form.addEventListener('reset', event => {
            handleFormReset(event);
        });
    });

    // Opacity slider logic: listen to slider changes and apply to preview area
    function setupOpacityControls() {
        const controls = document.querySelectorAll('.opacity-range');
        console.log('setupOpacityControls: found', controls.length, 'controls');
        controls.forEach(control => {
            if (control.dataset.initialized === '1') return;
            control.dataset.initialized = '1';
            const update = () => {
                const value = Number(control.value);
                // Locate percentage display in same control group
                const controlGroup = control.closest('.opacity-control');
                const display = controlGroup ? controlGroup.querySelector('.opacity-value') : null;
                if (display) display.textContent = value + '%';

                // Find preview area in parent panel
                const panelPreview = control.closest('.panel-preview');
                const preview = panelPreview ? panelPreview.querySelector('.preview-area') : null;
                if (preview) {
                    // Set checkerboard background opacity
                    preview.style.setProperty('--preview-bg-opacity', String(value / 100));
                    // Ensure preview area has gray background to show PNG transparency
                    preview.style.backgroundColor = '';
                    const img = preview.querySelector('img');
                    if (img) img.style.opacity = String(value / 100);
                }

                // Adjust slider track visuals
                const pct = value + '%';
                control.style.background = `linear-gradient(90deg, #007bff ${pct}, #d0d0d0 ${pct})`;
            };
            control.addEventListener('input', update);
            control.addEventListener('change', update);
            // Initialize
            update();
        });
    }

    setupOpacityControls();
    
    // Offset and scale control logic
    function setupTransformControls() {
        // Handle offset controls
        const offsetControls = document.querySelectorAll('.offset-range');
        offsetControls.forEach(control => {
            if (control.dataset.initialized === '1') return;
            control.dataset.initialized = '1';
            const update = () => {
                const value = Number(control.value);
                const axis = control.dataset.axis;
                
                // Update display value
                const controlGroup = control.closest('.transform-control');
                const display = controlGroup ? controlGroup.querySelector(`.offset-${axis}-value`) : null;
                if (display) display.textContent = value + 'px';
                
                // Apply transform to image
                applyTransformToPreview(control);
                
                // Adjust slider track visuals
                const pct = ((value + 100) / 200) * 100 + '%';
                control.style.background = `linear-gradient(90deg, #007bff ${pct}, #d0d0d0 ${pct})`;
            };
            control.addEventListener('input', update);
            control.addEventListener('change', update);
            update();
        });
        
        // Handle scale controls
        const scaleControls = document.querySelectorAll('.scale-range');
        scaleControls.forEach(control => {
            if (control.dataset.initialized === '1') return;
            control.dataset.initialized = '1';
            const update = () => {
                const value = Number(control.value);
                
                // Update display value
                const controlGroup = control.closest('.transform-control');
                const display = controlGroup ? controlGroup.querySelector('.scale-value') : null;
                if (display) display.textContent = value + '%';
                
                // Apply transform to image
                applyTransformToPreview(control);
                
                // Adjust slider track visuals
                const pct = ((value - 10) / 190) * 100 + '%';
                control.style.background = `linear-gradient(90deg, #007bff ${pct}, #d0d0d0 ${pct})`;
            };
            control.addEventListener('input', update);
            control.addEventListener('change', update);
            update();
        });
    }
    
    // Apply transform to preview image
    function applyTransformToPreview(controlElement) {
        // Find preview area in parent panel
        const panelPreview = controlElement.closest('.panel-preview');
        const preview = panelPreview ? panelPreview.querySelector('.preview-area') : null;
        if (!preview) return;
        
        const img = preview.querySelector('img');
        if (!img) return;
        
        // Get all transform values
        const isText = panelPreview.closest('#text-form') !== null;
        const offsetX = Number(document.getElementById(isText ? 'text-offset-x' : 'json-offset-x').value);
        const offsetY = Number(document.getElementById(isText ? 'text-offset-y' : 'json-offset-y').value);
        const scale = Number(document.getElementById(isText ? 'text-scale' : 'json-scale').value) / 100;
        
        // Apply transform
        img.style.transform = `translate(${offsetX}px, ${offsetY}px) scale(${scale})`;
    }
    
    setupTransformControls();
    // Initialize history panel and collapse button
    loadHistory();
    renderHistoryPanel();
    setupHistoryToggle();

    // Mode switch card logic (Plain Text / Tellraw JSON)
    const modeSwitch = document.getElementById('mode-switch');
    const textPanel = document.querySelector('#text-form');
    const jsonPanel = document.querySelector('#json-form');
    // wrapper panels that contain the h2 title
    const textWrapper = textPanel ? textPanel.closest('.panel') : null;
    const jsonWrapper = jsonPanel ? jsonPanel.closest('.panel') : null;
    const MODE_KEY = 'mcbe_text_impact_mode_v1';
    // applyMode: only update card selection state and save, but don't open panel
    function applyMode(mode) {
        modeSwitch.querySelectorAll('.mode-card').forEach(btn => {
            btn.setAttribute('aria-selected', btn.dataset.mode === mode ? 'true' : 'false');
        });
        try { localStorage.setItem(MODE_KEY, mode); } catch (e) {}
    }

    // showPanel: actually control panel display, only called on user interaction
    function showPanel(mode) {
        if (!textPanel || !jsonPanel) return;
        if (mode === 'tellraw') {
            if (textWrapper) textWrapper.style.display = 'none';
            if (jsonWrapper) jsonWrapper.style.display = '';
            textPanel.style.display = 'none';
            jsonPanel.style.display = '';
        } else {
            if (textWrapper) textWrapper.style.display = '';
            if (jsonWrapper) jsonWrapper.style.display = 'none';
            textPanel.style.display = '';
            jsonPanel.style.display = 'none';
        }
    }

    if (modeSwitch) {
    // Initialize: hide entire panel (including h2 and form), only restore last selected card style
    if (textWrapper) textWrapper.style.display = 'none';
    if (jsonWrapper) jsonWrapper.style.display = 'none';
    if (textPanel) textPanel.style.display = 'none';
    if (jsonPanel) jsonPanel.style.display = 'none';
        const saved = localStorage.getItem(MODE_KEY) || 'text';
        applyMode(saved);

        modeSwitch.addEventListener('click', (e) => {
            const btn = e.target.closest('.mode-card');
            if (!btn) return;
            const mode = btn.dataset.mode;
            applyMode(mode);
            showPanel(mode);
        });
    }
}

// If document is still loading, wait for DOMContentLoaded; otherwise initialize immediately
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initUI);
} else {
    initUI();
}