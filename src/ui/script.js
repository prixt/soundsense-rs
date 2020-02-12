'use strict';

function addSlider(channel_name) {
    let channels = document.getElementById("channels");
    channels.insertAdjacentElement(
        'beforeend',
        createSlider(channel_name)
    );
    let slider = document.getElementById(channel_name+"_slider");
    slider.addEventListener(
        /MSIE|Trident|Edge/.test(window.navigator.userAgent) ? 'change' : 'input',
        function() {
            external.invoke("change_volume:"+channel_name+":"+this.value);
        },
        false
    );
}

function createSlider(channel_name) {
    let slider = document.createElement("tr");
    slider.class="w3-row";
    slider.innerHTML=
    "<td class='w3-center' style='width:50px'><h4>"+channel_name+"</h4></td> \
    <td class='w3-rest'> \
        <input type='range' \
            name='"+channel_name+"_slider' \
            id='"+channel_name+"_slider' \
            min='0' \
            max='100' \
            value='100' \
        /> \
    </td>"
    return slider;
}

function clearSliders() {
    let channels = document.getElementById("channels");
    while (channels.firstChild) {
        channels.removeChild(channels.firstChild);
    }
}