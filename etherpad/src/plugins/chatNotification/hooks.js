import("etherpad.log");

function notificationStartup() {
 log.info("notification startup for chatNotification");
}

function notificationShutdown() {
 log.info("notification shutdown for chatNotification");
}

function modals(arg) {
  return arg.template.include('soundPlayer.ejs', undefined, ['chatNotification']);
}
