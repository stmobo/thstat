const { invoke, event } = window.__TAURI__;

function stage_name(stage) {
    switch (stage) {
    case 0: return "Stage 1";
    case 1: return "Stage 2";
    case 2: return "Stage 3";
    case 3: return "Stage 4";
    case 4: return "Stage 5";
    case 5: return "Stage 6";
    case 6: return "Extra Stage";
    case 7: return "Phantasm Stage";
    }
}

/**
 * 
 * @param {*} section 
 * @returns {Promise<string>}
 */
function section_name(section) {
    switch (section.type) {
    case "start": return Promise.resolve("Start");
    case "first_half": return Promise.resolve("First Half " + (section.seq + 1));
    case "midboss_nonspell": return Promise.resolve("Midboss Nonspell " + (section.seq + 1));
    case "midboss_spell": return invoke('format_spellcard', { 'spell': section.spell });
    case "second_half": return Promise.resolve("Second Half " + (section.seq + 1));
    case "pre_boss": return Promise.resolve("Pre-Boss");
    case "boss_nonspell": return Promise.resolve("Boss Nonspell " + (section.seq + 1));
    case "boss_spell": return invoke('format_spellcard', { 'spell': section.spell });
    default: return Promise.resolve("Unknown");
    }
}

/**
 * 
 * @param {*} loc 
 * @returns {Promise<string>}
 */
function format_stage_location(loc) {
    return section_name(loc.section).then((name) => {
        if (loc.section.type == "boss_spell" || loc.section.type == "midboss_spell") {
            return name;
        } else {
            return stage_name(loc.stage) + " " + name;
        }
    });
}

function difficulty_name(difficulty) {
    switch (difficulty) {
    case 0: return "Easy";
    case 1: return "Normal";
    case 2: return "Hard";
    case 3: return "Lunatic";
    case 4: return "Extra";
    case 5: return "Phantasm";
    default: return "Unknown";
    }
}

function character_name(character) {
    switch (character) {
    case 0: return "Reimu A";
    case 1: return "Reimu B";
    case 2: return "Marisa A";
    case 3: return "Marisa B";
    case 4: return "Sakuya A";
    case 5: return "Sakuya B";
    default: return "Unknown";
    }
}

/**
 * 
 * @param {*} ev 
 * @returns {Promise<string>}
 */
function format_game_event(ev) {
    switch (ev.event) {
    case "start_game":
        return format_stage_location(ev.location).then((location_name) => {
            if (ev.practice) {
                return "Started practice game as " + character_name(ev.character[1]) + " at " + location_name;
            } else {
                return "Started game as " + character_name(ev.character[1]) + " at " + location_name;
            }
        });
    case "end_game":
        return format_stage_location(ev.location).then((location_name) => {
            var ret = (ev.cleared ? "Cleared game at ": "Ended game at ") + location_name;

            if (ev.misses == 0 && ev.bombs == 0) {
                ret += " with no misses or bombs used";
            } else {
                if (ev.misses > 0) {
                    ret += " with " + (ev.misses == 1 ? "1 miss" : ev.misses + " misses");
                }

                if (ev.bombs > 0) {
                    if (ev.misses > 0 && ev.continues > 0) {
                        ret += ", ";
                    } else if (ev.misses > 0) {
                        ret += " and ";
                    } else {
                        ret += " with ";
                    }

                    ret += ev.bombs == 1 ? "1 bomb" : ev.bombs + " bombs";
                }

                if (ev.continues > 0) {
                    if (ev.misses > 0 && ev.bombs > 0) {
                        ret += ", and ";
                    } else if (ev.misses > 0) {
                        ret += " and ";
                    } else {
                        ret += " with ";
                    }

                    ret += ev.continues == 1 ? "1 continue used" : ev.continues + " continues used";
                }
            }

            return ret;
        });
    case "stage_cleared":
        return Promise.resolve("Cleared " + stage_name(ev.stage));
    case "enter_section": 
        return format_stage_location(ev.location).then((location_name) => {
            var ret = "Entering " + location_name + " with ";
            ret += (ev.lives == 1) ? "1 life, " : ev.lives + " lives, ";
            ret += (ev.bombs == 1) ? "1 bomb, and " : ev.bombs + " bombs, ";
            ret += ev.power + " power, and ";
            ret += (ev.continues == 1) ? "1 continue used" : ev.continues + " continues used";
            return ret;
        });
    case "extend":
        return format_stage_location(ev.location).then((location_name) => {
            return "Got extend at " + location_name
        });
    case "miss":
        return format_stage_location(ev.location).then((location_name) => {
            return "Missed at " + location_name
        });
    case "bomb":
        return format_stage_location(ev.location).then((location_name) => {
            return "Bombed at " + location_name
        });
    case "finish_spell":
        return invoke('format_spellcard', { 'spell': ev.spell }).then((formatted) => {
            return (ev.captured ? "Captured spell " : "Failed spell ") + formatted;
        });
    case "border_start":
        return format_stage_location(ev.location).then((location_name) => "Border started at " + location_name);
    case "border_end":
        return format_stage_location(ev.location).then((location_name) => {
            if (ev.broken) {
                return "Broke border at " + location_name;
            } else {
                return "Border ended at " + location_name;
            }
        });
    case "pause": return Promise.resolve("Game paused");
    case "unpause": return Promise.resolve("Game unpaused");
    default: return Promise.resolve("Unknown event type " + ev.event);
    }
}

/**
 * 
 * @param {string} elem_type 
 * @param {string} classes 
 * @returns {HTMLElement}
 */
function createElementWithClass(elem_type, classes) {
    var ret = document.createElement(elem_type);
    ret.className = classes;
    return ret;
}

function GameDisplay(disp_n, difficulty, character, practice, start_location, start_time) {
    this.container = createElementWithClass("div", "game-container");

    var subrow1 = createElementWithClass("div", "game-details-row game-summary-row");
    this.indexElem = createElementWithClass("div", "game-details-entry game-details-index");
    this.characterElem = createElementWithClass("div", "game-details-entry game-details-character");
    this.modeElem = createElementWithClass("div", "game-details-entry game-details-mode");
    this.locationElem = createElementWithClass("div", "game-details-entry game-details-location");
    this.resultElem = createElementWithClass("div", "game-details-entry game-details-result");
    this.locationCountElem = createElementWithClass("div", "game-details-entry game-details-location-count");
    this.durationElem = createElementWithClass("div", "game-details-entry game-details-duration");
    subrow1.appendChild(this.indexElem);
    subrow1.appendChild(this.characterElem);
    subrow1.appendChild(this.modeElem);
    subrow1.appendChild(this.locationElem);
    subrow1.appendChild(this.resultElem);
    subrow1.appendChild(this.locationCountElem);
    subrow1.appendChild(this.durationElem);

    this.indexElem.innerText = disp_n;

    this.missListContainer = createElementWithClass("div", "game-details-row game-location-list-container game-miss-container");
    var missListHeader = createElementWithClass("div", "game-location-list-header game-location-list-entry");
    missListHeader.innerText = "Misses";
    this.missListContainer.appendChild(missListHeader);

    this.missListElem = createElementWithClass("div", "game-location-list game-misses");
    this.missListContainer.appendChild(this.missListElem);

    this.bombListContainer = createElementWithClass("div", "game-details-row game-location-list-container game-bomb-container");
    var bombListHeader = createElementWithClass("div", "game-location-list-header game-location-list-entry");
    bombListHeader.innerText = "Bombs";
    this.bombListContainer.appendChild(bombListHeader);

    this.bombListElem = createElementWithClass("div", "game-location-list game-bombs");
    this.bombListContainer.appendChild(this.bombListElem);

    this.breakListContainer = createElementWithClass("div", "game-details-row game-location-list-container game-border-break-container");
    var breakListHeader = createElementWithClass("div", "game-location-list-header game-location-list-entry");
    breakListHeader.innerText = "Border Breaks";
    this.breakListContainer.appendChild(breakListHeader);

    this.breakListElem = createElementWithClass("div", "game-location-list game-border-breaks");
    this.breakListContainer.appendChild(this.breakListElem);
    
    this.container.appendChild(subrow1);
    this.container.appendChild(this.missListContainer);
    this.container.appendChild(this.bombListContainer);
    this.container.appendChild(this.breakListContainer);

    this.disp_n = disp_n;
    this.character = character;
    this.difficulty = difficulty;
    this.practice = practice;
    this.cleared = false;
    this.start_location = start_location;
    this.cur_location = start_location;
    this.start_time = start_time;
    this.end_time = null;
    this.seen_locations = 0;
    this.misses = [];
    this.bombs = [];
    this.breaks = [];
}

GameDisplay.updateLocationList = function(listElem, listContainer, locations) {
    if (locations.length > 0) {
        Promise.all(locations.map(format_stage_location)).then((locs) => {
            while (locs.length % 5 != 0) {
                locs.push("");
            }
    
            Element.prototype.replaceChildren.apply(listElem, locs.map((loc) => {
                var elem = createElementWithClass("div", "game-location-list-entry");
                elem.innerText = loc;
                return elem;
            }));

            listContainer.style.display = null;
        });
    } else {
        listContainer.style.display = "none";
    }
}

GameDisplay.prototype.update = function() {
    this.characterElem.innerText = character_name(this.character);

    if (this.practice) {
        this.modeElem.innerText = difficulty_name(this.difficulty) + " Practice";
    } else {
        this.modeElem.innerText = difficulty_name(this.difficulty);
    }

    if (this.start_location && this.cur_location) {
        Promise.all([
            format_stage_location(this.start_location),
            format_stage_location(this.cur_location)
        ]).then((locs) => {
            if (this.end_time) {
                if (this.cleared) {
                    if (this.practice) {
                        this.locationElem.innerText = "Cleared " + stage_name(this.start_location.stage);
                    } else {
                        this.locationElem.innerText = "All cleared";
                    }
                } else {
                    this.locationElem.innerText = "Ended at " + locs[1];
                }
            } else {
                this.locationElem.innerText = "Currently at " + locs[1];
            }
        })
    }

    GameDisplay.updateLocationList(this.missListElem, this.missListContainer, this.misses);
    GameDisplay.updateLocationList(this.bombListElem, this.bombListContainer, this.bombs);
    GameDisplay.updateLocationList(this.breakListElem, this.breakListContainer, this.breaks);

    var result = "";
    if (this.misses.length == 0 && this.bombs.length == 0 && this.breaks.length == 0) {
        result = "NNN";
    } else {
        if (this.misses.length > 0) result += this.misses.length + "M";
        if (this.bombs.length > 0) result += this.bombs.length + "B";
        if (this.breaks.length > 0) result += this.breaks.length + "BB";
    }

    var colorClass = "game-state-normal";
    if (this.end_time && !this.practice) {
        if (!this.cleared) result = "Failed";
        colorClass = this.cleared ? "game-state-cleared" : "game-state-failed";
    } else if (!this.end_time) {
        result = "Running (" + result + ")";
        colorClass = "game-state-running";
    }

    this.resultElem.innerText = result;

    this.container.classList.remove("game-state-normal", "game-state-running", "game-state-cleared", "game-state-failed");
    this.container.classList.add(colorClass);

    var duration = null;
    if (this.end_time) {
        duration = this.end_time.valueOf() - this.start_time.valueOf();
    } else {
        duration = Date.now() - this.start_time.valueOf();
    }
    
    var total_sec = Math.floor(duration / 1000);
    var m = Math.floor(total_sec / 60);
    var s = Math.floor(total_sec % 60);

    var tm_string = (
        ((m < 10) ? ("0" + m.toFixed(0)) : m.toFixed(0))
        + ":" + ((s < 10) ? ("0" + s.toFixed(0)) : s.toFixed(0))
    );

    var ms = duration % 1000;
    if (ms > 0) {
        if (ms < 10) {
            tm_string += ".00" + ms;
        } else if (ms < 100) {
            tm_string += ".0" + ms;
        } else {
            tm_string += "." + ms;
        }
    }

    this.durationElem.innerText = tm_string;
    this.locationCountElem.innerText = this.seen_locations;

    this.container.scrollIntoView(false);
}

function Game(start_event) {
    this.start_time = new Date(start_event.time * 1000);
    this.end_time = null;
    this.location = start_event.location;
    this.character = start_event.character[1];
    this.practice = start_event.practice;
    this.difficulty = start_event.difficulty;
    this.misses = [];
    this.bombs = [];
    this.breaks = [];
    this.locationsSeen = new Set();
    this.captured_spells = [];
    this.failed_spells = [];
    this.cleared = false;

    this.display = new GameDisplay(games.length + 1, this.difficulty, this.character, this.practice, this.location, this.start_time);
    this.update();
}

Game.prototype.update = function () {
    this.display.cur_location = this.location;
    this.display.cleared = this.cleared;
    this.display.end_time = this.end_time;
    this.display.seen_locations = this.locationsSeen.size;
    this.display.misses = this.misses.slice();
    this.display.bombs = this.bombs.slice();
    this.display.breaks = this.breaks.slice();
    this.display.update();
}

Game.prototype.addEvent = function (ev) {
    switch (ev.event) {
    case "enter_section":
        this.location = ev.location;
        format_stage_location(ev.location).then((s) => {
            this.locationsSeen.add(s);
            this.update()
        });
        break;
    case "end_game":
        this.end_time = new Date(ev.time * 1000);
        this.location = ev.location;
        this.cleared = ev.cleared;
        break;
    case "miss":
        this.misses.push(ev.location);
        break;
    case "bomb":
        this.bombs.push(ev.location);
        break;
    case "border_end":
        if (ev.broken) {
            this.breaks.push(ev.location);
        }
        break;
    default: break;
    }

    this.update();
}

var games = [];

var sess_start = null;

function add_log_row(ts, text) {
    if (!ts) {
        ts = new Date();
    } else {
        ts = new Date(ts * 1000);
    }

    var row = document.createElement("div");
    row.className = "event-row";

    var date_span = document.createElement("span");
    if (games.length > 0 && !games[games.length - 1].end_time) {
        let sess_start = games[games.length - 1].start_time;
        let secs = (ts.valueOf() - sess_start.valueOf()) / 1000.0;
        date_span.innerText = ((secs > 0) ? "+" : "") + secs.toFixed(3);
    } else {
        date_span.innerText = ts.toISOString();
    }

    date_span.className = "event-time";
    row.appendChild(date_span);

    var text_span = document.createElement("span");
    text_span.innerText = text;
    text_span.className = "event-text";
    row.appendChild(text_span);

    var log = document.getElementById("event-log")
    log.appendChild(row);
    while (log.childNodes.length > 50 && log.firstChild) {
        log.firstChild.remove();
    }

    log.scrollTop = log.scrollHeight;
    
    return row;
}

var unlisten_events = null;

document.addEventListener("DOMContentLoaded", () => {
    unlisten_events = [
        event.listen("game-event", (ev) => {
            try {
                if (ev.payload.event == "start_game") {
                    var new_game = new Game(ev.payload);
                    games.push(new_game);
                    document.getElementById("game-list").appendChild(new_game.display.container);
                    new_game.display.container.scrollIntoView(false);
                } else if (games.length > 0) {
                    var cur_game = games[games.length - 1];
                    cur_game.addEvent(ev.payload);
                }
    
                format_game_event(ev.payload).then((formatted) => {
                    add_log_row(ev.payload.time, formatted);
                });
            } catch (e) {
                console.error(e);
            }
        }),
        event.listen("game-attached", (ev) => {
            add_log_row(null, "Attached to PID " + ev.payload);
            sess_start = null;
        }),
        event.listen("game-detached", (ev) => {
            add_log_row(null, "Waiting for PCB...");
            sess_start = null;
        }),
        event.listen("error", (ev) => {
            add_log_row(null, "Error: " + ev.payload);
        })
    ];

    invoke('init_events', {});
})
