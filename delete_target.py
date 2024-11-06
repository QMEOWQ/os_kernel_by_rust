import os
import shutil

# 获取当前目录
current_directory = os.getcwd()
# print(current_directory)

# 遍历当前目录下的所有子目录
# subdir 是一个字符串，表示当前遍历到的目录的路径。
# dirs 是一个列表，包含当前目录下的所有子目录的名称。
# files 是一个列表，包含当前目录下的所有文件名称。
for subdir, dirs, files in os.walk(current_directory):
    for dir in dirs:
        # 检查目录名是否为'target'
        if dir == 'target':
            # 构建完整的目录路径
            target_dir = os.path.join(subdir, dir)
            # 删除目录及其所有内容
            shutil.rmtree(target_dir)
            print(f"Deleted '{target_dir}'")

print("All 'target' folders have been deleted.")