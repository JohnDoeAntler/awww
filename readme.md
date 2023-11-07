daemon
  read the folder
    - spawn webview
    - reload webview

cli
  communicate with the daemon
  reload webview
  show specified webview
  hide specified webview

folder structure
  .config
    awww
      [layer position]/
        [screen_num]/
          [applications]
            ...
            index.html

3 threads
  main thread
    for listening socket
    sending the received command from socket to ui thread

  ui thread
    for rendering the webview
    for receiving

  webserver thread
    for serving the static files

main thread <-> ui thread operations:
  quit (window)
  reload (quit all windows and restart)
  --
  create () (layer, application_name, id: string)
  close () id
  show (application window) (layer, application_name)
  hide (application window)
  reload uri (webview)
  --
  dispatch (webview)
  get data (webview)

