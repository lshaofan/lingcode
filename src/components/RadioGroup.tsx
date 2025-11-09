import React from 'react';

export interface RadioOption {
  value: string;
  label: string;
  description?: string;
  disabled?: boolean;
}

interface RadioGroupProps {
  name: string;
  value: string;
  onChange: (value: string) => void;
  options: RadioOption[];
  className?: string;
}

export const RadioGroup: React.FC<RadioGroupProps> = ({
  name,
  value,
  onChange,
  options,
  className = '',
}) => {
  return (
    <div className={`space-y-2 ${className}`}>
      {options.map((option) => (
        <label
          key={option.value}
          className={`
            flex items-start p-3 rounded-lg border cursor-pointer
            transition-colors duration-200
            ${
              value === option.value
                ? 'border-green-500 bg-green-50'
                : 'border-gray-200 hover:border-gray-300'
            }
            ${option.disabled ? 'opacity-50 cursor-not-allowed' : ''}
          `}
        >
          <input
            type="radio"
            name={name}
            value={option.value}
            checked={value === option.value}
            onChange={(e) => !option.disabled && onChange(e.target.value)}
            disabled={option.disabled}
            className="mt-1 h-4 w-4 text-green-500 focus:ring-green-500"
          />
          <div className="ml-3 flex-1">
            <div className="font-medium text-gray-900">{option.label}</div>
            {option.description && (
              <div className="text-sm text-gray-500 mt-1">{option.description}</div>
            )}
          </div>
        </label>
      ))}
    </div>
  );
};
