"use strict";

let is_windows = null;
let channels = null;
function addSlider(channel_name) {
    channels.insertAdjacentElement(
        'beforeend',
        createSlider(channel_name)
    );
    let slider = document.getElementById(channel_name+"_slider");
    slider.addEventListener(is_windows?'change':'input',function(){
            external.invoke("change_volume:"+channel_name+":"+this.value);
        },
        false
    );
    let skip_button = document.getElementById(channel_name+"_skip_button");
    skip_button.addEventListener('click',function(){
            external.invoke("skip_current_sound:"+channel_name);
        },
        false
    )
}
function createSlider(channel_name) {
    let slider = document.createElement("div");
    slider.className="w3-cell-row w3-border-bottom";
    slider.insertAdjacentHTML(
        'afterbegin',
        "<div class='w3-cell w3-cell-middle w3-center w3-padding-small' style='width:10%;min-width:90px'>"+
            "<h4>"+channel_name+"</h4>"+
        "</div>"+
        "<button class='w3-button w3-round w3-cell w3-cell-middle w3-grey w3-small' style='width:2%;min-width:10px'"+
            "id='"+channel_name+"_skip_button'>"+
            "<h4>&#x23ED;</h4>"+
        "</button>"+
        "<div class='w3-cell w3-cell-middle w3-rest w3-container w3-padding-small'>"+
            "<input type='range' id='"+channel_name+"_slider'"+
                "min='0' max='100' value='100'>"+
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