<script>
  import { invoke } from '@tauri-apps/api/core'
  import { exit } from '@tauri-apps/plugin-process'
  import { listen } from '@tauri-apps/api/event'
  import { onMount } from 'svelte'

  async function terminateApp() {
    await exit(0)
  }

  let activeTab = 'bot'
  let config = {
    allowed_roles: [],
    allowed_users: []
  }
  let saving = false
  let message = ''
  let building = false
  let buildProgress = 0
  let buildMessage = ''

  const tabs = [
    { id: 'bot', label: 'BOT SETTINGS', code: 'R-25' },
    { id: 'installation', label: 'INSTALLATION', code: 'I-47' },
    { id: 'startup', label: 'STARTUP', code: 'S-89' },
    { id: 'decoy', label: 'DECOY', code: 'D-12' },
    { id: 'auth', label: 'AUTH', code: 'A-33' },
    { id: 'features', label: 'FEATURES', code: 'F-56' },
    { id: 'build', label: 'BUILD', code: 'B-78' },
  ]

  const installPaths = [
    { value: 1, label: '%LOCALAPPDATA%\\Packages' },
    { value: 2, label: '%LOCALAPPDATA%\\Microsoft\\WindowsApps' },
    { value: 3, label: '%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Autofill\\4.0.1.27\\' },
    { value: 4, label: '%APPDATA%\\Microsoft\\Windows\\Themes' },
    { value: 5, label: '%APPDATA%\\Microsoft\\Templates' },
    { value: 6, label: '%LOCALAPPDATA%\\Microsoft\\Windows\\INetCache' },
    { value: 7, label: '%LOCALAPPDATA%\\Microsoft\\Windows\\WebCache' },
  ]

  let loading = true

  onMount(async () => {
    try {
      config = await invoke('read_existing_config')
      loading = false
    } catch (error) {
      console.error('Failed to load config:', error)
      config = await invoke('get_default_config')
      loading = false
    }
  })

  async function saveConfig() {
    saving = true
    message = ''
    try {
      const result = await invoke('save_config_file', { config })
      message = result
      setTimeout(() => message = '', 5000)
    } catch (error) {
      message = `Error: ${error}`
    } finally {
      saving = false
    }
  }

  function addToList(list, value) {
    if (value && !config[list].includes(value)) {
      config[list] = [...config[list], value]
    }
  }

  function removeFromList(list, index) {
    config[list] = config[list].filter((_, i) => i !== index)
  }

  async function buildBot() {
    console.log('Build button clicked!')
    building = true
    buildProgress = 0
    buildMessage = 'Initializing build...'

    try {
      console.log('Setting up build progress listener...')
      const unlisten = await listen('build-progress', (event) => {
        console.log('Build progress event received:', event.payload)
        buildProgress = event.payload.progress
        buildMessage = event.payload.message
      })

      console.log('Calling build_bot command...')

      const result = await invoke('build_bot')
      console.log('Build command result:', result)

      unlisten()

      buildMessage = result

      setTimeout(() => {
        building = false
      }, 3000)
    } catch (error) {
      console.error('Build error:', error)
      buildMessage = `Build failed: ${error}`
      building = false
    }
  }
</script>

<div class="min-h-screen bg-cyber-black">
  {#if loading}
    <div class="flex items-center justify-center min-h-screen bg-cyber-black">
      <div class="text-center space-y-6 p-8 bg-cyber-black-light border-4 border-cyber-red cyber-clip">
        <div class="w-20 h-20 bg-cyber-red flex items-center justify-center font-cyber-title font-bold text-cyber-black cyber-clip-simple animate-spin text-4xl mx-auto">
          K
        </div>
        <p class="text-cyber-red text-xl font-bold font-cyber-title uppercase tracking-wider animate-glitch">Loading configuration...</p>
      </div>
    </div>
  {:else}
  <!-- Custom Title Bar -->
  <div class="bg-cyber-black border-b-4 border-cyber-red select-none">
    <div class="flex items-center justify-between px-4 py-2">
      <div class="flex items-center gap-3 flex-1">
        <div class="w-8 h-8 bg-cyber-red flex items-center justify-center font-cyber-title font-bold text-cyber-black cyber-clip-simple">
          K
        </div>
        <div>
          <h1 class="text-lg font-extrabold text-cyber-red font-cyber-title uppercase tracking-wider">
            KURINIUM // BUILDER
          </h1>
          <p class="text-cyber-green text-[10px] font-semibold tracking-wide">[ DISCORD BOT CONFIGURATION SYSTEM ]</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <div class="px-6 py-1 bg-cyber-red text-cyber-black font-bold text-[10px] cyber-clip-simple">
          v0.2.6
        </div>

        <!-- Terminate Button -->
        <button
          on:click={terminateApp}
          class="w-10 h-8 ml-4 bg-cyber-black-light hover:bg-cyber-red-darker text-cyber-red hover:text-white transition-colors flex items-center justify-center border-2 border-cyber-gray hover:border-cyber-red"
          style="clip-path: polygon(0px 0px, 100% 0px, 100% 100%, 0px 100%);"
        >
          <span class="text-2xl leading-none">×</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Main Layout with Vertical Tabs -->
  <div class="flex h-[calc(100vh-68px)]">
    <!-- Vertical Sidebar -->
    <aside class="w-64 bg-cyber-black border-r-4 border-cyber-red overflow-y-auto">
      <nav class="p-4 space-y-2">
        {#each tabs as tab}
          <button
            class="w-full text-left px-4 py-3 text-sm font-bold transition-all relative uppercase tracking-wider border-l-4 {activeTab === tab.id ? 'bg-cyber-red text-cyber-black border-cyber-yellow' : 'bg-cyber-black-light text-cyber-red hover:bg-cyber-gray border-cyber-gray hover:border-cyber-red'}"
            on:click={() => activeTab = tab.id}
            style="clip-path: polygon(0px 0px, 100% 0px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 0px 100%);"
          >
            <div class="flex items-center justify-between">
              <span class="relative z-10">{tab.label}</span>
              <span class="text-[10px] font-mono {activeTab === tab.id ? 'text-cyber-black' : 'text-cyber-yellow'}">{tab.code}</span>
            </div>
          </button>
        {/each}
      </nav>
    </aside>

    <!-- Content Area -->
    <main class="flex-1 overflow-y-auto p-6">
    {#if activeTab === 'bot'}
      <div class="space-y-6">
        <div class="bg-cyber-black-light p-8 space-y-6 border-4 border-cyber-red cyber-clip">
          <div class="flex items-center gap-3 mb-6">
            <div class="w-2 h-8 bg-cyber-red"></div>
            <h2 class="text-2xl font-bold text-cyber-red uppercase font-cyber-title tracking-wider">BOT CONFIGURATION</h2>
            <div class="flex-1 h-0.5 bg-gradient-to-r from-cyber-red to-transparent ml-4"></div>
          </div>
          
          <div class="space-y-3">
            <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">
              <span class="text-cyber-neon">▸</span> DISCORD TOKEN
            </label>
            <input
              type="text"
              bind:value={config.discord_token}
              class="cyber-input"
              placeholder="ENTER YOUR DISCORD BOT TOKEN"
            />
          </div>

          <div class="space-y-3">
            <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">
              <span class="text-cyber-red">▸</span> GUILD ID
            </label>
            <input
              type="text"
              bind:value={config.guild_id}
              class="cyber-input"
              placeholder="YOUR DISCORD SERVER ID"
            />
          </div>

          <div class="grid grid-cols-2 gap-6">
            <div class="space-y-3">
              <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">
                <span class="text-cyber-red">▸</span> BOT PREFIX
              </label>
              <input
                type="text"
                bind:value={config.bot_prefix}
                class="cyber-input"
                placeholder="."
              />
            </div>

            <div class="space-y-3">
              <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">
                <span class="text-cyber-red">▸</span> MAX FILE SIZE (MB)
              </label>
              <input
                type="number"
                step="0.1"
                bind:value={config.max_file_size_mb}
                class="cyber-input"
              />
            </div>
          </div>

          <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
            <input
              type="checkbox"
              id="show_console"
              bind:checked={config.show_console}
              class="cyber-checkbox"
            />
            <label for="show_console" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
              SHOW CONSOLE WINDOW
            </label>
          </div>
        </div>
      </div>
    {/if}

    {#if activeTab === 'installation'}
      <div class="bg-cyber-black-light p-8 border-4 border-cyber-red cyber-clip">
        <div class="flex items-center gap-3 mb-6">
          <div class="w-2 h-8 bg-cyber-red" style="box-shadow: 0 0 10px rgba(255, 0, 60, 0.8);"></div>
          <h2 class="text-2xl font-bold text-cyber-red uppercase font-cyber-title tracking-wider cyber-text-glow">Installation Path</h2>
          <div class="flex-1 h-0.5 bg-gradient-to-r from-cyber-red to-transparent ml-4"></div>
        </div>
        
        <div class="space-y-3">
          <label class="block text-sm font-semibold text-gray-200 mb-2 flex items-center gap-2">
            <span class="text-primary">●</span>
            Select Installation Location
          </label>
          <select
            bind:value={config.installation_path}
            class="cyber-select"
          >
            {#each installPaths as path}
              <option value={path.value}>{path.label}</option>
            {/each}
          </select>
          <p class="text-xs text-cyber-silver font-bold uppercase tracking-wide mt-3 flex items-center gap-2">
            <span class="text-cyber-red">⚠️</span>
            Choose where the bot will be installed on the target system
          </p>
        </div>
      </div>
    {/if}

    {#if activeTab === 'startup'}
      <div class="bg-cyber-black-light p-8 border-4 border-cyber-red cyber-clip space-y-6">
        <div class="flex items-center gap-3 mb-6">
          <div class="w-2 h-8 bg-cyber-red" style="box-shadow: 0 0 10px rgba(255, 0, 60, 0.8);"></div>
          <h2 class="text-2xl font-bold text-cyber-red uppercase font-cyber-title tracking-wider cyber-text-glow">Startup Configuration</h2>
          <div class="flex-1 h-0.5 bg-gradient-to-r from-cyber-red to-transparent ml-4"></div>
        </div>
        
        <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
          <input
            type="checkbox"
            id="startup_enabled"
            bind:checked={config.startup_enabled}
            class="cyber-checkbox"
          />
          <label for="startup_enabled" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
            Enable Startup Task
          </label>
        </div>

        {#if config.startup_enabled}
          <div>
            <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Task Name</label>
            <input
              type="text"
              bind:value={config.startup_task_name}
              class="cyber-input"
            />
          </div>

          <div class="space-y-3">
            <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
              <input
                type="checkbox"
                id="startup_on_logon"
                bind:checked={config.startup_on_logon}
                class="cyber-checkbox"
              />
              <label for="startup_on_logon" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
                Run on User Logon
              </label>
            </div>

            <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
              <input
                type="checkbox"
                id="startup_highest_privileges"
                bind:checked={config.startup_highest_privileges}
                class="cyber-checkbox"
              />
              <label for="startup_highest_privileges" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
                Run with Highest Privileges
              </label>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    {#if activeTab === 'decoy'}
      <div class="bg-cyber-black-light p-8 border-4 border-cyber-red cyber-clip space-y-4">
        <div class="flex items-center gap-3 mb-6">
          <div class="w-2 h-8 bg-cyber-red" style="box-shadow: 0 0 10px rgba(255, 0, 60, 0.8);"></div>
          <h2 class="text-2xl font-bold text-cyber-red uppercase font-cyber-title tracking-wider cyber-text-glow">Fake Error Configuration</h2>
          <div class="flex-1 h-0.5 bg-gradient-to-r from-cyber-red to-transparent ml-4"></div>
        </div>
        
        <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
          <input
            type="checkbox"
            id="decoy_enabled"
            bind:checked={config.decoy_enabled}
            class="cyber-checkbox"
          />
          <label for="decoy_enabled" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
            Enable Fake Error Message
          </label>
        </div>

        {#if config.decoy_enabled}
          <div>
            <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Title</label>
            <input
              type="text"
              bind:value={config.decoy_title}
              class="cyber-input"
            />
          </div>

          <div>
            <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Message</label>
            <textarea
              bind:value={config.decoy_message}
              rows="4"
              class="cyber-textarea"
            ></textarea>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Icon</label>
              <select
                bind:value={config.decoy_icon}
                class="cyber-select"
              >
                <option value="Error">Error</option>
                <option value="Warning">Warning</option>
                <option value="Info">Info</option>
                <option value="Question">Question</option>
              </select>
            </div>

            <div>
              <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Buttons</label>
              <select
                bind:value={config.decoy_buttons}
                class="cyber-select"
              >
                <option value="Ok">Ok</option>
                <option value="OkCancel">Ok / Cancel</option>
                <option value="YesNo">Yes / No</option>
              </select>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    {#if activeTab === 'auth'}
      <div class="bg-cyber-black-light p-8 border-4 border-cyber-red cyber-clip space-y-4">
        <div class="flex items-center gap-3 mb-6">
          <div class="w-2 h-8 bg-cyber-red" style="box-shadow: 0 0 10px rgba(255, 0, 60, 0.8);"></div>
          <h2 class="text-2xl font-bold text-cyber-red uppercase font-cyber-title tracking-wider cyber-text-glow">Authentication Settings</h2>
          <div class="flex-1 h-0.5 bg-gradient-to-r from-cyber-red to-transparent ml-4"></div>
        </div>
        
        <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
          <input
            type="checkbox"
            id="auth_all"
            bind:checked={config.auth_all}
            class="cyber-checkbox"
          />
          <label for="auth_all" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
            Allow Everyone (No Authentication)
          </label>
        </div>

        {#if !config.auth_all}
          <div class="space-y-4">
            <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
              <input
                type="checkbox"
                id="auth_roles"
                bind:checked={config.auth_roles}
                class="cyber-checkbox"
              />
              <label for="auth_roles" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
                Authenticate by Roles
              </label>
            </div>

            {#if config.auth_roles}
              <div>
                <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Allowed Role IDs</label>
                <div class="space-y-3">
                  {#each config.allowed_roles as role, i}
                    <div class="flex gap-2">
                      <input
                        type="text"
                        value={role}
                        readonly
                        class="flex-1 cyber-input"
                      />
                      <button
                        on:click={() => removeFromList('allowed_roles', i)}
                        class="cyber-button-remove"
                      >
                        Remove
                      </button>
                    </div>
                  {/each}
                  <div class="flex gap-2">
                    <input
                      type="text"
                      id="new_role"
                      class="flex-1 cyber-input"
                      placeholder="Enter role ID"
                    />
                    <button
                      on:click={() => {
                        const input = document.getElementById('new_role')
                        addToList('allowed_roles', input.value)
                        input.value = ''
                      }}
                      class="cyber-button-secondary"
                    >
                      Add
                    </button>
                  </div>
                </div>
              </div>
            {/if}

            <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
              <input
                type="checkbox"
                id="auth_user"
                bind:checked={config.auth_user}
                class="cyber-checkbox"
              />
              <label for="auth_user" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
                Authenticate by Users
              </label>
            </div>

            {#if config.auth_user}
              <div>
                <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Allowed User IDs</label>
                <div class="space-y-3">
                  {#each config.allowed_users as user, i}
                    <div class="flex gap-2">
                      <input
                        type="text"
                        value={user}
                        readonly
                        class="flex-1 cyber-input"
                      />
                      <button
                        on:click={() => removeFromList('allowed_users', i)}
                        class="cyber-button-remove"
                      >
                        Remove
                      </button>
                    </div>
                  {/each}
                  <div class="flex gap-2">
                    <input
                      type="text"
                      id="new_user"
                      class="flex-1 cyber-input"
                      placeholder="Enter user ID"
                    />
                    <button
                      on:click={() => {
                        const input = document.getElementById('new_user')
                        addToList('allowed_users', input.value)
                        input.value = ''
                      }}
                      class="cyber-button-secondary"
                    >
                      Add
                    </button>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    {#if activeTab === 'features'}
      <div class="space-y-6">
        <div class="bg-cyber-black-light p-8 border-4 border-cyber-red cyber-clip space-y-4">
          <div class="flex items-center gap-3 mb-6">
            <div class="w-2 h-8 bg-cyber-red"></div>
            <h2 class="text-2xl font-bold text-cyber-red uppercase font-cyber-title tracking-wider">Keep Active</h2>
            <div class="flex-1 h-0.5 bg-gradient-to-r from-cyber-red to-transparent ml-4"></div>
          </div>
          
          <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
            <input
              type="checkbox"
              id="keep_active_enabled"
              bind:checked={config.keep_active_enabled}
              class="cyber-checkbox"
            />
            <label for="keep_active_enabled" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
              Enable Keep Active
            </label>
          </div>

          {#if config.keep_active_enabled}
            <div>
              <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Interval (seconds)</label>
              <input
                type="number"
                bind:value={config.keep_active_interval}
                class="cyber-input"
              />
            </div>
          {/if}
        </div>

        <div class="bg-cyber-black-light p-8 border-4 border-cyber-red cyber-clip space-y-4">
          <div class="flex items-center gap-3 mb-6">
            <div class="w-2 h-8 bg-cyber-red"></div>
            <h2 class="text-2xl font-bold text-cyber-red uppercase font-cyber-title tracking-wider">WiFi Monitor</h2>
            <div class="flex-1 h-0.5 bg-gradient-to-r from-cyber-red to-transparent ml-4"></div>
          </div>
          
          <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
            <input
              type="checkbox"
              id="wifi_monitor_enabled"
              bind:checked={config.wifi_monitor_enabled}
              class="cyber-checkbox"
            />
            <label for="wifi_monitor_enabled" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
              Enable WiFi Monitor
            </label>
          </div>

          {#if config.wifi_monitor_enabled}
            <div class="space-y-4">
              <div>
                <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Check Interval (milliseconds)</label>
                <input
                  type="number"
                  bind:value={config.wifi_check_interval_ms}
                  class="cyber-input"
                />
              </div>

              <div>
                <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Re-enable Delay (seconds)</label>
                <input
                  type="number"
                  bind:value={config.wifi_re_enable_delay_seconds}
                  class="cyber-input"
                />
              </div>

              <div class="flex items-center gap-4 p-5 bg-cyber-black border-4 border-cyber-gray hover:border-cyber-red transition-colors cyber-clip-simple">
                <input
                  type="checkbox"
                  id="wifi_block_user_input"
                  bind:checked={config.wifi_block_user_input}
                  class="cyber-checkbox"
                />
                <label for="wifi_block_user_input" class="text-sm font-bold text-cyber-red cursor-pointer flex-1 uppercase tracking-wide">
                  Block User Input During Re-enable
                </label>
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    {#if activeTab === 'build'}
      <div class="bg-cyber-black-light p-8 border-4 border-cyber-red cyber-clip space-y-4">
        <div class="flex items-center gap-3 mb-6">
          <div class="w-2 h-8 bg-cyber-red" style="box-shadow: 0 0 10px rgba(255, 0, 60, 0.8);"></div>
          <h2 class="text-2xl font-bold text-cyber-red uppercase font-cyber-title tracking-wider cyber-text-glow">Build Information</h2>
          <div class="flex-1 h-0.5 bg-gradient-to-r from-cyber-red to-transparent ml-4"></div>
        </div>
        <p class="text-xs text-cyber-silver font-bold uppercase tracking-wide mb-4">These details will make your bot appear as a legitimate application</p>
        
        <div>
          <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">File Name</label>
          <input
            type="text"
            bind:value={config.build_file_name}
            class="cyber-input"
          />
        </div>

        <div>
          <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Product Name</label>
          <input
            type="text"
            bind:value={config.build_product_name}
            class="cyber-input"
          />
        </div>

        <div>
          <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Description</label>
          <input
            type="text"
            bind:value={config.build_description}
            class="cyber-input"
          />
        </div>

        <div>
          <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">Company Name</label>
          <input
            type="text"
            bind:value={config.build_company_name}
            class="cyber-input"
          />
        </div>

        <div>
          <label class="block text-xs font-bold text-cyber-silver mb-2 uppercase tracking-widest">File Version</label>
          <input
            type="text"
            bind:value={config.build_file_version}
            class="cyber-input"
          />
        </div>

        <!-- Build Button -->
        <div class="mt-8">
          <button
            on:click={buildBot}
            disabled={building}
            class="w-full cyber-button text-xl {building ? 'animate-glitch' : ''}"
          >
            {#if building}
              <span class="flex items-center justify-center gap-3">
                <div class="w-6 h-6 bg-black flex items-center justify-center font-cyber-title font-bold text-cyber-red text-sm animate-spin">
                  K
                </div>
                BUILDING BOT...
              </span>
            {:else}
              BUILD KURINIUM BOT
            {/if}
          </button>

          {#if building || buildProgress > 0}
            <div class="mt-6 space-y-3">
              <!-- Progress Bar -->
              <div class="relative h-12 bg-cyber-black border-4 border-cyber-red overflow-hidden cyber-clip-simple">
                <div
                  class="absolute inset-0 bg-gradient-to-r from-cyber-red to-cyber-red-light transition-all duration-500 ease-out"
                  style="width: {buildProgress}%;"
                ></div>
                <div class="absolute inset-0 flex items-center justify-center">
                  <span class="text-cyber-yellow font-bold text-lg uppercase tracking-wider cyber-text-glow-yellow z-10">
                    {buildProgress}%
                  </span>
                </div>
              </div>

              <!-- Build Message -->
              {#if buildMessage}
                <div class="px-6 py-3 bg-cyber-black-light border-4 border-cyber-yellow text-cyber-yellow font-bold text-sm uppercase tracking-wide text-center cyber-clip-simple">
                  {buildMessage}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Save Button -->
    <div class="mt-10 p-8 bg-cyber-black-light border-4 border-cyber-red" style="clip-path: polygon(0px 25px, 26px 0px, calc(60% - 25px) 0px, 60% 25px, 100% 25px, 100% calc(100% - 10px), calc(100% - 15px) calc(100% - 10px), calc(80% - 10px) calc(100% - 10px), calc(80% - 15px) calc(100% - 0px), 10px calc(100% - 0px), 0% calc(100% - 10px));">
      <div class="flex items-center gap-6">
        <button
          on:click={saveConfig}
          disabled={saving}
          class="cyber-button text-xl {saving ? 'animate-glitch' : ''}"
        >
          {#if saving}
            <span class="flex items-center gap-3">
              <div class="w-6 h-6 bg-black flex items-center justify-center font-cyber-title font-bold text-cyber-red text-sm animate-spin">
                K
              </div>
              SAVING CONFIGURATION...
            </span>
          {:else}
            SAVE CONFIGURATION
          {/if}
        </button>

        {#if message}
          <div class="px-6 py-3 font-bold text-sm {message.startsWith('Error') ? 'bg-cyber-red-darker text-white' : 'bg-cyber-red text-cyber-black'} cyber-clip-simple border-2 border-cyber-red-light">
            {message}
          </div>
        {/if}
      </div>
    </div>
    </main>
  </div>
  {/if}
</div>

<style>
  select option {
    background-color: #1a1a1a;
    color: #ff003c;
    font-weight: 700;
  }
</style>



















