"""Grava results/<run>/ambiente.json: hardware, SO, energia, estado do Smart App Control, versões das ferramentas,
GPU preferida por executável e temperatura (quando o Windows expõe). Uso: python -I harness/ambiente.py <saida.json> [--perfil X] [--versao nome=valor ...]"""
import json, os, platform, subprocess, sys, datetime


def ps(cmd):
    r = subprocess.run(["powershell", "-NoProfile", "-Command", cmd], capture_output=True, text=True, encoding="utf-8", errors="replace")
    return r.stdout.strip()


def pj(cmd):
    t = ps(cmd + " | ConvertTo-Json -Compress")
    try:
        return json.loads(t) if t else None
    except Exception:
        return t


def cmd_ver(c):
    try:
        return subprocess.run(c, capture_output=True, text=True, timeout=20).stdout.strip().splitlines()[0]
    except Exception:
        return None


def coletar(perfil="", versoes=None):
    cpu = pj("Get-CimInstance Win32_Processor | Select Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed")
    gpu = pj("Get-CimInstance Win32_VideoController | Select Name,DriverVersion")
    ram = ps("[math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB,1)")
    os_ = pj("Get-CimInstance Win32_OperatingSystem | Select Caption,Version,BuildNumber")
    plano = ps("powercfg /getactivescheme")
    bateria = ps("(Get-CimInstance Win32_Battery | Select -First 1 BatteryStatus).BatteryStatus")  # 2 = na tomada
    sac = ps("(Get-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\CI\\Policy' -ErrorAction SilentlyContinue).VerifiedAndReputablePolicyState")
    temp = ps("try { (Get-CimInstance -Namespace root/wmi MSAcpi_ThermalZoneTemperature -ErrorAction Stop | Select -First 1).CurrentTemperature/10-273.15 } catch { '' }")
    defender = ps("try { (Get-MpComputerStatus).RealTimeProtectionEnabled } catch { '' }")
    return {
        "quando": datetime.datetime.now().astimezone().isoformat(),
        "perfil": perfil,
        "so": os_,
        "cpu": cpu,
        "ram_gb": ram,
        "gpus": gpu,
        "plano_energia": plano,
        "bateria_status_2_tomada": bateria,
        "smart_app_control_estado": sac or "n/d (0=desligado, 1=ligado, 2=avaliacao)",
        "temperatura_cpu_c": temp or "n/d (o Windows nao expoe neste equipamento)",
        "defender_tempo_real": defender,
        "ferramentas": {
            "rustc": cmd_ver(["rustc", "--version"]),
            "cargo": cmd_ver(["cargo", "--version"]),
            "node": cmd_ver(["node", "--version"]),
            "python": platform.python_version(),
            "dotnet": cmd_ver(["dotnet", "--version"]),
            "flutter": cmd_ver(["flutter", "--version"]),
        },
        "versoes_app": versoes or {},
    }


if __name__ == "__main__":
    a = sys.argv[1:]
    perfil = a[a.index("--perfil") + 1] if "--perfil" in a else ""
    versoes = {}
    for i, x in enumerate(a):
        if x == "--versao":
            k, v = a[i + 1].split("=", 1)
            versoes[k] = v
    os.makedirs(os.path.dirname(os.path.abspath(a[0])), exist_ok=True)
    json.dump(coletar(perfil, versoes), open(a[0], "w", encoding="utf-8"), ensure_ascii=False, indent=2)
    print("ambiente gravado em", a[0])
