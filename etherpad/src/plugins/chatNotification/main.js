import("etherpad.log");
import("plugins.chatNotification.hooks");
import("plugins.chatNotification.static.js.main");

function chatNotificationInit() {
    this.hooks = ['notificationStartup', 'notificationShutdown', 'modals'];
 this.client = new main.chatNotificationPluginInit();
 this.description = 'Plays sound when new chat message is received';
 this.notificationStartup = hooks.notificationStartup;
 this.notificationShutdown = hooks.notificationShutdown;
 this.modals = hooks.modals;
 //`this.handlePath = hooks.handlePath;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing chatNotification");
}

function uninstall() {
 log.info("Uninstalling chatNotification");
}

