local release_tag = "wms"

settings.load("wms_task.settings")
local machine_task = settings.get("task")
local machine_version = settings.get("version")

function wait_for_updates()
    -- todo: use github actions releases webhook
    sleep(10)

    return false
end

function latest_release()
    -- todo: get latest release url
    return ""
end

function download_and_install(url)
    -- todo: use github actions releases webhook
end

download_and_install(latest_release())

local machine_task_pid = -1

while true do
    if machine_task_pid == -1 then
        machine_task_pid = multishell.launch({}, "tasks/" .. machine_task .. ".lua")
    end

    if wait_for_updates() then
        if machine_task_pid ~= -1 then
            os.queueEvent("killtask")

            while multishell.setFocus(machine_task_pid) do
                sleep(0.1)
            end

            machine_task_pid = -1
        end

        download_and_install()
    end
end
