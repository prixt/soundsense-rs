"use strict";

let is_windows = null;
let channels = null;

function addSlider(channel_name) {
    channels.insertAdjacentElement(
        'beforeend',
        createSlider(channel_name)
    );
    document.getElementById(channel_name+"_slider")
        .addEventListener(is_windows?'change':'input',function(){
                window.webkit.messageHandlers.external.postMessage("change_volume:"+channel_name+":"+this.value);
            },
            false
        );
    document.getElementById(channel_name+"_skip_button")
        .addEventListener('click',function(){
                window.webkit.messageHandlers.external.postMessage("skip_current_sound:"+channel_name);
            },
            false
        );
    document.getElementById(channel_name+"_play_pause_button")
        .addEventListener('click',function(){
                window.webkit.messageHandlers.external.postMessage("play_pause:"+channel_name);
            },
            false
        );
}
function createSlider(channel_name) {
    let slider = document.createElement("div");
    slider.className="w3-cell-row w3-border-bottom";
    slider.insertAdjacentHTML(
        'afterbegin',
        "<div class='w3-cell w3-cell-middle w3-center w3-padding-small overlay-container' style='width:10%;min-width:90px'"+
            "id='"+channel_name+"_head'>"+
            "<div class='overlay-content w3-container w3-padding-small w3-animate-opacity w3-grey w3-center'>"+
                "<span>Threshold</span><br>"+
                "<select id='"+channel_name+"_selector' onchange='thresholdSelect(\""+channel_name+"\",this.value)'>"+
                    "<option value='4'>Everything</option>"+
                    "<option value='3'>Fluff</option>"+
                    "<option value='2'>Important</option>"+
                    "<option value='1'>Critical</option>"+
                    "<option value='0'>Nothing</option>"+
                "</select>"+
            "</div>"+
            "<h4>"+channel_name+"</h4>"+
        "</div>"+
        "<div class='w3-cell w3-cell-middle w3-rest w3-container w3-padding-small'>"+
            "<input type='range' id='"+channel_name+"_slider'"+
                "min='0' max='100' value='100'>"+
        "</div>"+
        "<div class='w3-cell w3-cell-middle w3-center w3-small w3-padding-small' style='width:2%;min-width:10px;'>"+
            "<div class='w3-button w3-block w3-round w3-small w3-padding-small'"+
                "title='Skip "+channel_name+"'"+
                "id='"+channel_name+"_skip_button'>"+
                "&#x23ED;"+
            "</div>"+
            "<div class='w3-button w3-block w3-round w3-small w3-padding-small'"+
                "title='Play/Pause "+channel_name+"'"+
                "id='"+channel_name+"_play_pause_button'>"+
                "&#x23EF;"+
            "</div>"+
        "</div>"
    );
    return slider;
}
function setSliderValue(channel_name, value) {
    let slider = document.getElementById(channel_name+"_slider");
    if (slider != null) slider.value = value;
}
function clearSliders() {
    while (channels.firstChild)
        channels.removeChild(channels.firstChild);
}
function setSliderHead(channel_name, is_paused) {
    let slider_head = document.getElementById(channel_name+"_head");
    let slider = document.getElementById(channel_name+"_slider");
    if (is_paused) {
        slider_head.classList.add("w3-opacity-max");
        slider.classList.add("w3-opacity-max");
    }
    else {
        slider_head.classList.remove("w3-opacity-max");
        slider.classList.remove("w3-opacity-max");
    }
}

let alerts_footer = null;
let alerts = null;
function addAlert(name, color, text) {
    removeAlert(name);
    let new_alert = createAlert(name, color, text);
    alerts[name] = new_alert;
    alerts_footer.insertAdjacentElement('beforeend', new_alert);
    if (alerts_footer.childElementCount > 10)
        removeAlert(alerts_footer.firstChild.name);
}
function removeAlert(name) {
    let alert = document.getElementById("alert_"+name);
    if (alert != null) {
        alerts_footer.removeChild(alert);
        alerts.delete(name);
    }
}
function createAlert(name, color, text) {
    let alert=document.createElement("div");
    alert.name = name;
    alert.id="alert_"+name;
    alert.className="w3-bar w3-animate-bottom w3-"+color;
    alert.style.cssText="padding: 2px 15px 2px 15px;";
    alert.innerHTML=text;
    alert.timer = 4.0;

    let cross = document.createElement("span");
    cross.className="w3-closebtn";
    cross.setAttribute("onclick", "removeAlert('"+name+"')");
    cross.innerHTML="&times;";

    alert.insertAdjacentElement('afterbegin',cross);
    
    return alert;
}

let error_footer = null;
function addError(name, text) {
    let new_error = createError(name, text);
    error_footer.insertAdjacentElement('afterbegin', new_error);
}
function removeError(id) {
    let error = document.getElementById(id);
    if (error != null) {
        error_footer.removeChild(error);
    }
}
function createError(name, text) {
    let error=document.createElement("div");
    error.name=name;
    error.id="error_"+name+toString(Math.floor(Math.random()*100000));
    error.className="w3-bar w3-animate-bottom w3-red";
    error.style.cssText="padding: 10px 15px 10px 15px;";
    error.innerHTML="<h3>"+name+"</h3><p>"+text+"</p>";

    let cross = document.createElement("span");
    cross.className="w3-closebtn";
    cross.setAttribute("onclick", "removeError('"+error.id+"')");
    cross.innerHTML="&times;";

    error.insertAdjacentElement('afterbegin',cross);

    return error;
}

function thresholdSelect(channel_name, value) {
    window.webkit.messageHandlers.external.postMessage("change_threshold:"+channel_name+":"+value);

}

function main() {
    channels = document.getElementById('channels');
    is_windows = /MSIE|Trident|Edge/.test(window.navigator.userAgent);
    alerts_footer = document.getElementById('alerts');
    error_footer = document.getElementById('errors');
    alerts = new Map();
    
    let prev = null;
    function step(now) {
        let dt = (prev!=null) ? (now-prev)*0.001 : 0.0;
        prev = now;
        for (let key in alerts) {
            let alert = alerts[key];
            alert.timer -= dt;
            if (alert.timer <= 1.0) alert.style.opacity = alert.timer;
            if (alert.timer <= 0.0) removeAlert(alert.name);
        }
        window.requestAnimationFrame(step);
    }
    window.requestAnimationFrame(step);
}
